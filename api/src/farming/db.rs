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
pub struct TotalsByDex {
        #[sql_type="Varchar"]
        pub name: String,
        #[sql_type="Json"]
        pub meta: Value,
        #[sql_type="Double"]
        pub tvl: f64,
}

#[derive(Serialize,Deserialize,QueryableByName,Clone,Debug)]
pub struct TotalsByChain {
        #[sql_type="Varchar"]
        pub name: String,
        #[sql_type="Json"]
        pub meta: Value,
        #[sql_type="Double"]
        pub tvl: f64,
}

#[derive(Serialize,Deserialize,QueryableByName,Clone,Debug)]
pub struct FarmsView {
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

impl FarmsView {

    pub async fn totals_by_dex(
        conn: &PgConnection,
    ) -> Result<Vec<TotalsByDex>> {
        diesel::sql_query("SELECT * FROM totals_by_dex;") 
            .get_results(conn)
            .map_err(|e|e.into())
    }

    pub async fn totals_by_chain(
        conn: &PgConnection,
    ) -> Result<Vec<TotalsByChain>> {
        diesel::sql_query("SELECT * FROM totals_by_chain;") 
            .get_results(conn)
            .map_err(|e|e.into())
    }

    pub async fn get(
        conn: &PgConnection,
    ) -> Result<Vec<Self>> {
        diesel::sql_query("SELECT * FROM farms_for_api;") 
            .get_results(conn)
            .map_err(|e|e.into())
    }
}

