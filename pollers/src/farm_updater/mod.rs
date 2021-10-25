use crate::DbPool;
use bigdecimal::BigDecimal;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sql_types::*,
};
use tokio::time::{sleep, Duration};

use crate::db_types::*;
use ethcontract::web3::{
    self,
    contract::{Contract, Options},
    types::*,
};

use crate::schema::{farms, gton_price, staking};

pub type Web3Instance = web3::Web3<ethcontract::Http>;

#[derive(Clone, Debug, AsChangeset)]
#[table_name = "staking"]
pub struct StakesUpdater {
    tvl: f64,
    total_locked: BigDecimal,
    alloc_point: f64,
    apy: f64,
    total_rewards: BigDecimal,
}

#[derive(Clone, Debug, AsChangeset)]
#[table_name = "farms"]
pub struct FarmsUpdater {
    tvl: f64,
    total_locked: BigDecimal,
    alloc_point: f64,
    apy: f64,
    total_rewards: BigDecimal,
}

#[derive(Clone, Debug, QueryableByName)]
pub struct Staking {
    #[sql_type = "BigInt"]
    pub farm_id: i64,
    #[sql_type = "BigInt"]
    pub farm_index: i64,
    #[sql_type = "Varchar"]
    pub coingeco_id: String,
    #[sql_type = "Varchar"]
    pub rpc_url: String,
    #[sql_type = "ChainType"]
    pub chain_type: ChainTypeEnum,
    #[sql_type = "Varchar"]
    pub farm_address: String,
    #[sql_type = "Numeric"]
    pub blocks_in_year: BigDecimal,
}

#[derive(Clone, Debug, QueryableByName)]
pub struct Farms {
    #[sql_type = "BigInt"]
    pub farm_id: i64,
    #[sql_type = "BigInt"]
    pub farm_index: i64,
    #[sql_type = "Double"]
    pub pool_tvl: f64,
    #[sql_type = "Numeric"]
    pub pool_lp_supply: BigDecimal,
    #[sql_type = "Varchar"]
    pub coingeco_id: String,
    #[sql_type = "Varchar"]
    pub rpc_url: String,
    #[sql_type = "ChainType"]
    pub chain_type: ChainTypeEnum,
    #[sql_type = "Varchar"]
    pub farm_address: String,
    #[sql_type = "Numeric"]
    pub blocks_in_year: BigDecimal,
}

impl Farms {
    pub fn get_lp_farms_data(conn: &PgConnection) -> Vec<Farms> {
        diesel::sql_query("select * from farms_from_poller;")
            .get_results(conn)
            .unwrap()
    }

    pub fn get_staking_farms_data(conn: &PgConnection) -> Vec<Staking> {
        diesel::sql_query("select * from staking_from_poller;")
            .get_results(conn)
            .unwrap()
    }
}

pub fn prepare_amount(reserve: U256, dec: i64) -> f64 {
    reserve.to_f64_lossy() / 10_f64.powf(dec as f64)
}

pub fn create_instance(rpc_url: &str) -> Web3Instance {
    let http = web3::transports::Http::new(String::from(rpc_url).as_str())
        .expect("error creating web3 instance");
    web3::Web3::new(http)
}

pub async fn get_total_supply(web3: &Web3Instance, pool: Address) -> f64 {
    let contract = Contract::from_json(web3.eth(), pool, include_bytes!("./abi/erc20.json"))
        .expect("error contract creating");
    let res = contract
        .query("totalSupply", (), None, Options::default(), None)
        .await
        .unwrap();
    prepare_amount(res, 18)
}

pub async fn get_from_farm(
    web3: &Web3Instance,
    farm_address: Address,
    farm_index: U256,
) -> (U256, U256, U256, U256, U256) {
    let contract = Contract::from_json(web3.eth(), farm_address, include_bytes!("./abi/farm.json"))
        .expect("error contract creating");
    let res: (U256, U256, U256, U256, U256) = contract
        .query("getPoolData", farm_index, None, Options::default(), None)
        .await
        .unwrap();
    // ap tap tc vl
    (res.0, res.1, res.2, res.3, res.4)
}

pub fn calc_lp_price(tvl: f64, total_supply: f64) -> f64 {
    tvl / total_supply
}

pub fn calc_lp_liquidity(lp_price: f64, lp_locked: f64) -> f64 {
    lp_price * lp_locked
}

pub fn calculate_apy(
    total_locked: f64,
    gton_price: f64,
    amount_per_block: i64,
    blocks_in_year: i64,
) -> f64 {
    // total earn per year / total locked on contract
    // all values are compatible to dollar value
    (blocks_in_year * amount_per_block) as f64 * gton_price / total_locked * 100 as f64
}

pub async fn get_gton_price(conn: &PgConnection) -> f64 {
    gton_price::table
        .order_by(gton_price::price_hour.desc())
        .select(gton_price::price)
        .get_result::<f64>(conn)
        .unwrap()
}

pub struct FarmExtractor {
    pool: DbPool,
}

pub fn string_to_h160(string: String) -> ethcontract::H160 {
    ethcontract::H160::from_slice(String::from(string).as_bytes())
}

impl FarmExtractor {
    pub fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").expect("missing db url");
        let manager = ConnectionManager::<PgConnection>::new(&db_url);
        let pool = Pool::builder().build(manager).expect("pool build");
        FarmExtractor { pool }
    }
    pub async fn update_stakes(&self) -> () {
        loop {
            let farms: Vec<Staking> = Farms::get_staking_farms_data(&self.pool.get().unwrap());
            let gton_price = get_gton_price(&self.pool.get().unwrap()).await;
            for farm in farms {
                let web3 = create_instance(&farm.rpc_url);
                // ap tap tc vl
                //
                let farm_index: U256 = farm.farm_index.into();
                let (alloc_point, total_alloc_point, total_claimed, value_locked, mint_per_block) =
                    get_from_farm(
                        &web3,
                        farm.farm_address.parse().unwrap(),
                        farm.farm_index.into(),
                    )
                    .await;
                let tvl = gton_price * value_locked.to_f64_lossy();
                let apy = calculate_apy(
                    tvl,
                    gton_price,
                    mint_per_block.as_u64() as i64,
                    farm.blocks_in_year.to_string().parse().unwrap(),
                );
                let share = alloc_point.to_f64_lossy() / total_alloc_point.to_f64_lossy();
                let query = FarmsUpdater {
                    tvl,
                    total_locked: value_locked.to_string().parse().unwrap(),
                    alloc_point: share,
                    apy,
                    total_rewards: total_claimed.to_string().parse().unwrap(),
                };
            }
            sleep(Duration::from_secs((15) as u64)).await;
        }
    }
    pub async fn update_farms(&self) -> () {
        loop {
            let farms: Vec<Farms> = Farms::get_lp_farms_data(&self.pool.get().unwrap());
            let gton_price = get_gton_price(&self.pool.get().unwrap()).await;
            for farm in farms {
                let web3 = create_instance(&farm.rpc_url);
                // ap tap tc vl
                //
                let farm_index: U256 = farm.farm_index.into();
                let (alloc_point, total_alloc_point, total_claimed, value_locked, mint_per_block) =
                    get_from_farm(
                        &web3,
                        farm.farm_address.parse().unwrap(),
                        farm.farm_index.into(),
                    )
                    .await;
                let lp_price: f64 = calc_lp_price(
                    farm.pool_tvl,
                    farm.pool_lp_supply.to_string().parse().unwrap(),
                );
                let tvl = lp_price * value_locked.to_f64_lossy();
                let apy = calculate_apy(
                    tvl,
                    gton_price,
                    mint_per_block.as_u64() as i64,
                    farm.blocks_in_year.to_string().parse().unwrap(),
                );
                let share = alloc_point.to_f64_lossy() / total_alloc_point.to_f64_lossy();
                let query = FarmsUpdater {
                    tvl,
                    total_locked: value_locked.to_string().parse().unwrap(),
                    alloc_point: share,
                    apy,
                    total_rewards: total_claimed.to_string().parse().unwrap(),
                };
            }
            sleep(Duration::from_secs((15) as u64)).await;
        }
    }
}
