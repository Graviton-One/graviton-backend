use crate::DbPool;
use actix_web::{HttpResponse, web::{self, Query}};
use super::db::{
    StakingView,
};
use serde::{Serialize,Deserialize};
use actix_web_dev::error::{
    Result,
};
pub fn staking_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/staking", web::get().to(get_staking_page_info));
}

pub async fn get_staking_page_info(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    Ok(HttpResponse::Ok().json(json!(
        {
            "achiei": StakingView::get(&conn).await?,
        }
    )))
}

