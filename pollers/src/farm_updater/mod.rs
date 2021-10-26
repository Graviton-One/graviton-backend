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
}

#[derive(Clone, Debug, AsChangeset)]
#[table_name = "farms"]
pub struct FarmsUpdater {
    tvl: f64,
    total_locked: BigDecimal,
    alloc_point: f64,
    apy: f64,
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
    #[sql_type = "Varchar"]
    pub wormhole_address: String,
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
    #[sql_type = "Varchar"]
    pub wormhole_address: String,
}

impl Farms {
    pub fn get_lp_farms_data(conn: &PgConnection) -> Vec<Farms> {
        diesel::sql_query("select * from farms_for_poller;")
            .get_results(conn)
            .unwrap()
    }

    pub fn get_staking_farms_data(conn: &PgConnection) -> Vec<Staking> {
        diesel::sql_query("select * from staking_for_poller;")
            .get_results(conn)
            .unwrap()
    }

    pub fn set_stake_data(stake_id: i64, query: &StakesUpdater, conn: &PgConnection) -> () {
        diesel::update(staking::table)
            .filter(staking::id.eq(stake_id))
            .set(query)
            .execute(conn)
            .unwrap();
    }

    pub fn set_farm_data(farm_id: i64, query: &FarmsUpdater, conn: &PgConnection) -> () {
        diesel::update(farms::table)
            .filter(farms::id.eq(farm_id))
            .set(query)
            .execute(conn)
            .unwrap();
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

pub async fn get_gton_to_relict_nums(web3: &Web3Instance, 
    wormhole: Address) -> (U256,U256) {
    let contract = Contract::from_json(
        web3.eth(), wormhole, include_bytes!("./abi/wormhole.json"))
        .expect("error contract creating");
  println!("holw addr: {:?}:{:?}",wormhole,web3);
    let res: (U256, U256) = contract
        .query("getValues", (), None, Options::default(), None)
        .await
        .unwrap();
    res
}

pub async fn get_from_farm(
    web3: &Web3Instance,
    farm_address: Address,
    farm_index: U256,
) -> (U256, U256, U256, U256) {
    let contract = Contract::from_json(
        web3.eth(), farm_address, include_bytes!("./abi/farm.json"))
        .expect("error contract creating");
  println!("farm addr: {:?}:{:?}:{:?}",farm_address,farm_index,web3);
    let res: (U256, U256, U256, U256) = contract
        .query("getPoolData", farm_index, None, Options::default(), None)
        .await
        .unwrap();
    // ap tap vl
    (res.0, res.1, res.2,res.3)
}

pub fn calc_lp_price(tvl: f64, total_supply: f64) -> f64 {
    tvl / total_supply
}

pub fn calc_lp_liquidity(lp_price: f64, lp_locked: f64) -> f64 {
    lp_price * lp_locked
}

pub fn calculate_apy(
    total_locked: f64,
    relict_price: f64,
    token_per_year: f64
) -> f64 {
    // total earn per year / total locked on contract
    // all values are compatible to dollar value
    token_per_year * relict_price / total_locked / 10f64.powf(18f64) * 100 as f64
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
            let farms: Vec<Staking> = 
                Farms::get_staking_farms_data(&self.pool.get().unwrap());
            let gton_price = get_gton_price(&self.pool.get().unwrap()).await;
            for farm in farms {
                let web3 = create_instance(&farm.rpc_url);
                // ap tap tc vl
                let (alloc_point, total_alloc_point, value_locked, mint_per_block) =
                    get_from_farm(
                        &web3,
                        farm.farm_address.parse().unwrap(),
                        farm.farm_index.into(),
                    )
                    .await;
                let tvl = gton_price * value_locked.to_f64_lossy() / 10f64.powf(18f64);
                let share = alloc_point.to_f64_lossy() / total_alloc_point.to_f64_lossy();
                let nums = get_gton_to_relict_nums(&web3, 
                        farm.wormhole_address.parse().unwrap()).await;
                let gton_to_relict_price = 
                    nums.0.to_f64_lossy() * gton_price / nums.1.to_f64_lossy();
                let apy = calculate_apy(
                    tvl,
                    gton_to_relict_price,
                    share *
                    mint_per_block.as_u64() as f64 *
                    farm.blocks_in_year.to_string().parse::<f64>().unwrap(),
                );
                let query = StakesUpdater {
                    tvl,
                    total_locked: value_locked.to_string().parse().unwrap(),
                    alloc_point: share,
                    apy,
                };
                Farms::set_stake_data(farm.farm_id, &query, &self.pool.get().unwrap());
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
                let (alloc_point, total_alloc_point, value_locked, mint_per_block) =
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
                let tvl = lp_price * value_locked.to_f64_lossy() / 10f64.powf(18f64);
                let share = alloc_point.to_f64_lossy() / total_alloc_point.to_f64_lossy();
                let nums = get_gton_to_relict_nums(&web3, 
                        farm.wormhole_address.parse().unwrap()).await;
                let gton_to_relict_price = 
                    nums.0.to_f64_lossy() * gton_price / nums.1.to_f64_lossy();
                let apy = calculate_apy(
                    tvl,
                    gton_to_relict_price,
                    share *
                    mint_per_block.as_u64() as f64 *
                    farm.blocks_in_year.to_string().parse::<f64>().unwrap(),
                );
                println!("apy: {:?}\n tvl: {:?}",apy,tvl);
                let query = FarmsUpdater {
                    tvl,
                    total_locked: value_locked.to_string().parse().unwrap(),
                    alloc_point: share,
                    apy,
                };
                Farms::set_farm_data(farm.farm_id, &query, &self.pool.get().unwrap());
            }
            sleep(Duration::from_secs((15) as u64)).await;
        }
    }
}
