use diesel::prelude::*;
use actix_web_dev::error::{
    Result,
};
use serde::{
    Serialize,
    Deserialize,
};
use diesel::sql_types::*;
use serde_json::Value;

#[derive(Serialize,Deserialize,QueryableByName,Clone,Debug)]
pub struct StakingView {
        #[sql_type="BigInt"]
        pub chain_id: i64,
        #[sql_type="Varchar"]
        pub chain_name: String,
        #[sql_type="Varchar"]
        pub chain_type: String,
        #[sql_type="Varchar"]
        pub explorer_url: String,
        #[sql_type="Varchar"]
        pub rpc_url: String,
        #[sql_type="Json"]
        pub chain_meta: Value,
        #[sql_type="BigInt"]
        pub farm_id: i64,
        #[sql_type="Varchar"]
        pub farm_address: String,
        #[sql_type="Double"]
        pub farm_tvl: f64,
        #[sql_type="Double"]
        pub apy: f64,
        #[sql_type="Varchar"]
        pub total_locked: String,
        #[sql_type="Varchar"]
        pub wormhole_address: String,
        #[sql_type="BigInt"]
        pub farm_index: i64,
        #[sql_type="Double"]
        pub alloc_point: f64,
        #[sql_type="Timestamp"]
        pub farm_last_update: chrono::NaiveDateTime,
}

impl StakingView {

    pub async fn get(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        diesel::sql_query("SELECT * FROM staking_for_api;") 
            .get_results(conn)
            .map_err(|e|e.into())
    }
}

