use diesel::{
    prelude::*,
    result::Error,
};
use crate::schema::pollers;

pub fn save(
    id: i64,
    num: i64,
    conn: &PgConnection, 
) -> Result<(),Error> {
    diesel::update(pollers::table)
        .filter(pollers::id.eq(id))
        .set(pollers::block_num.eq(num))
        .execute(conn)
        .map(|v|())
}

pub fn load(
    id: i64,
    conn: &PgConnection, 
) -> Result<i64,Error> {
    pollers::table
        .filter(pollers::id.eq(id))
        .select(pollers::block_num)
        .get_result::<i64>(conn)
}
