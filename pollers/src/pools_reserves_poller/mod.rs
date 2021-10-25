use std::error::Error;
use bigdecimal::BigDecimal;
use diesel::{
    sql_types::*,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use std::str::FromStr;
use hex::ToHex;
use tokio::time::{
    sleep,
  Duration,
};
use crate::DbPool;
use std::sync::Arc;
use serde_json::Value;
use crate::schema::{
    pools,
    chains,
    dexes,
};
use crate::db_types::ChainTypeEnum;

use ethcontract::web3::{
    self,
    contract::{Contract, Options},
    types::*,
};
use diesel::sql_types::*;
pub type Web3Instance = web3::Web3<ethcontract::Http>;
use crate::db_types::*;

#[derive(Debug,Clone, QueryableByName)]
pub struct Pools {
    #[sql_type="BigInt"]
    pub pool_id: i64,
    #[sql_type="Varchar"]
    pub token_a_address: String,
    #[sql_type="Varchar"]
    pub token_b_address: String,
    #[sql_type="Varchar"]
    pub pool_address: String,
    #[sql_type="ChainType"]
    pub chain_type: ChainTypeEnum,
    #[sql_type="Varchar"]
    pub chain_name: String,
    #[sql_type="Varchar"]
    pub rpc_url: String,
    #[sql_type="Varchar"]
    pub coingeco_id: String,
}

#[derive(Debug, Clone, AsChangeset)]
#[table_name="pools"]
pub struct PoolsUpdate {
    pub tvl: f64,
    pub reserve_a: BigDecimal,
    pub reserve_b: BigDecimal,
    pub lp_total_supply: BigDecimal,
    pub last_updated: chrono::NaiveDateTime,
}

pub fn create_instance(rpc_url: &str) -> Web3Instance {
    let http = web3::transports::Http::new(String::from(rpc_url).as_str())
        .expect("error creating web3 instance");
    web3::Web3::new(http)
}

impl Pools {

    fn get_pools(conn: DbPool) -> Vec<Pools> {
        diesel::sql_query("select * from pools_for_poller")
            .get_results::<Pools>(&conn.get().unwrap())
            .unwrap()
    }
    
    fn set_pool_data(
        pool_id: i64,
        query: &PoolsUpdate, 
        conn: DbPool,
    ) -> () {
        diesel::update(pools::table)
            .filter(pools::id.eq(pool_id))
            .set(query)
            .execute(&conn.get().unwrap())
            .unwrap();
    }

}


async fn retrieve_token<T: web3::Transport>(
    contract: &Contract<T>, 
    property: &str
) -> Result<Address, web3::contract::Error> {
    contract
        .query(property, (), None, Options::default(), None).await
}

pub async fn get_token_price(chain: &String, address: &String) -> f64 {
    let url = String::from("https://api.coingecko.com/api/v3/simple/token_price/") + 
        &chain + "?contract_addresses=" + &address + "&vs_currencies=usd";
    println!("{}", url);
    println!("{}", address);
    let resp: Value = reqwest::get(url)
    .await
    .unwrap()
    .json::<Value>()
    .await
    .unwrap();
    let v = resp[address.to_lowercase()]["usd"].as_f64();
    // we need to handle bad response from coingecko
    if v.is_none() {
        println!("set to 1");
        1 as f64
    } else {
        v.unwrap()
    }
    
}

pub async fn get_decimals(address: &Address, web3: Web3Instance) -> i64 {
    let contract = Contract::from_json(
        web3.eth(),
        *address,
        include_bytes!("./abi/erc20.json"),
    ).expect("error contract creating");
    contract
        .query("decimals", (), None, Options::default(), None).await.unwrap()
}

pub async fn get_pool_reserves(
    pool_address: &str,
    web3: Web3Instance,
) -> Result<(U256,U256), Box<dyn Error>> {
    let contract = Contract::from_json(
        web3.eth(),
        pool_address.parse().unwrap(),
        include_bytes!("./abi/pancakeV2pair.json"),
    ).expect("error contract creating");
    let (token_a_reserves, token_b_reserves, _): (U256, U256, U256) = contract
        .query("getReserves", (), None, Options::default(), None).await?;
    use std::str::FromStr;
    Ok((   
        token_a_reserves,
        token_b_reserves,
    ))
}

fn hex_to_string<T: ToHex>(h: T) -> String {
    "0x".to_owned() + &h.encode_hex::<String>()
}

pub struct PoolsExtractor {
    pool: Pool<ConnectionManager<PgConnection>>,
}

pub fn string_to_h160(string: &str) -> ethcontract::H160 {
    ethcontract::H160::from_slice(String::from(string).as_bytes())
}

pub async fn get_total_supply(web3: &Web3Instance, pool: Address) -> BigDecimal {
    let contract = Contract::from_json(
        web3.eth(),
        pool,
        include_bytes!("./abi/erc20.json"),
    ).expect("error contract creating");
    let res: U256 = contract
        .query("totalSupply", (), None, Options::default(), None).await.unwrap();
    BigDecimal::from_str(
        &res.to_string()
    ).unwrap()
}

pub fn prepare_reserve(reserve: U256, dec: i64) -> f64 {
    reserve.to_f64_lossy() / 10_f64.powf(dec as f64)
}

impl PoolsExtractor {
    pub fn new() -> Self {
        let manager = ConnectionManager::<PgConnection>::new(
            std::env::var("DATABASE_URL").expect("missing db url"),
        );
        let pool = Pool::builder().build(manager).expect("pool build");
        PoolsExtractor {pool}
    }
    pub async fn get_pool_data(&self) -> () {
        loop {
        let pools = Pools::get_pools(self.pool.clone());
        for pool in pools {
            let web3 = create_instance(&pool.rpc_url);
            let reserves = get_pool_reserves(
                &pool.pool_address, 
                web3.clone()).await.expect("Error getting pool reserves");
            let price_a: f64 = get_token_price(
                &pool.coingeco_id, 
                &hex_to_string(&pool.token_a_address)).await;
            let price_b: f64 = get_token_price(
                &pool.coingeco_id, 
                &hex_to_string(&pool.token_b_address)).await;
            let dec_a = get_decimals(
                &pool.token_a_address.parse().unwrap(), 
                web3.clone())
                .await;
            let dec_b = get_decimals(
                &pool.token_b_address.parse().unwrap(),
                web3.clone())
                .await;
            let reserve_a = prepare_reserve(reserves.0, dec_a);
            let reserve_b = prepare_reserve(reserves.1, dec_b);
            let tvl = price_a * reserve_a + price_b * reserve_b;
            let total_supply = get_total_supply(&web3, 
                pool.pool_address.parse().unwrap()).await;
            let query = PoolsUpdate {
                tvl,
                reserve_a: BigDecimal::from_str(&reserves.0.to_string()).unwrap(),
                reserve_b: BigDecimal::from_str(&reserves.1.to_string()).unwrap(),
                lp_total_supply: total_supply,
                last_updated: chrono::Utc::now().naive_utc(),
            };
            Pools::set_pool_data(pool.pool_id, &query, self.pool.clone());
        }
        //sleep(Duration::from_secs((15) as u64)).await;
    }
    }
}
