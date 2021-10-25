#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_json;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub mod db_types;
pub mod farm_updater;
pub mod pollers;
pub mod pools_reserves_poller;
pub mod schema;
