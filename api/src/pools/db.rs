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
pub struct PoolsView {
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
        #[sql_type="Json"]
        pub dex_meta: Value,
        #[sql_type="Varchar"]
        pub dex_name: String,
        #[sql_type="BigInt"]
        pub pool_id: i64,
        #[sql_type="Varchar"]
        pub pool_address: String,
        #[sql_type="Varchar"]
        pub token_a_address: String,
        #[sql_type="Varchar"]
        pub token_b_address: String,
        #[sql_type="Varchar"]
        pub reserve_a: String,
        #[sql_type="Varchar"]
        pub reserve_b: String,
        #[sql_type="Double"]
        pub tvl: f64,
        #[sql_type="Varchar"]
        pub lp_total_supply: String,
        #[sql_type="Json"]
        pub pool_meta: Value,
        #[sql_type="Timestamp"]
        pub last_update: chrono::NaiveDateTime,
}

impl PoolsView {

    pub async fn get(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        diesel::sql_query("SELECT * FROM pools_for_api;") 
            .get_results(conn)
            .map_err(|e|e.into())
    }
}

