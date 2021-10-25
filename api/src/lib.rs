#[macro_use]
extern crate diesel;
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
pub mod farming;
