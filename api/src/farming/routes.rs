use crate::DbPool;
use actix_web::{HttpResponse, web::{self, Query}};
use super::db::{
    Achievements,
};
use serde::{Serialize,Deserialize};
use actix_web_dev::error::{
    Result,
};
pub fn users_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users")
        .route("/portfolio", web::get().to(get_portfolio))
    );
}

#[derive(Serialize,Deserialize)]
pub struct UserAddress {
    address: String,
    chain: String,
}

pub async fn get_portfolio(
    data: Query<UserAddress>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    let ach = Achievements::get(
        data.address.as_str(),
        data.chain.as_str(),
        &conn).await?;
    let totals = TotalValues::get(&conn).await?;
    let split_by_sources = UsersValues::get(
        data.address.as_str(), 
        data.chain.as_str(),
        &conn).await?;
    Ok(HttpResponse::Ok().json(json!(
                {
                    "achievements": ach,
                    "total_values": totals,
                    "split_by_sources": split_by_sources
                }
    )))
}



