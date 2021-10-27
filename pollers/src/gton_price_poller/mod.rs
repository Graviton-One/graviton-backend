use tokio::time::{
        sleep,
      Duration,
};
use serde_json::Value;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use crate::schema::gton_price;
use crate::DbPool;

pub struct PricePoller {
    pool: DbPool,
}

impl PricePoller {
    pub fn new(
    ) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(
            std::env::var("DATABASE_URL").expect("missing db url"));
        let pool = Pool::builder().build(manager).expect("pool build");
        PricePoller {pool}
    }

    pub async fn run(&self) {
        loop {
            let url = String::from("https://api.coingecko.com/api/v3/coins/graviton");
            let resp: Value = reqwest::get(url)
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
            let v = resp["market_data"]["current_price"]["usd"].as_f64().unwrap();
            diesel::insert_into(gton_price::table)
                .values(gton_price::price.eq(v))
                .execute(&self.pool.get().unwrap())
                .unwrap();
            sleep(Duration::from_secs((3600) as u64)).await;
        }
    }
}

