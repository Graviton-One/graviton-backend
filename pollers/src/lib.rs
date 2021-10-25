#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_json;
use diesel::{
    PgConnection,
    r2d2::{
        ConnectionManager,
        Pool,
    },
};
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub mod schema;
pub mod pools_reserves_poller;
pub mod gton_price_poller;
pub mod pollers;
pub mod db_types;
pub mod farm_updater;
