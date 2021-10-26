use crate::DbPool;
use actix_web::{HttpResponse, web::{self, Query}};
use super::db::{
    PoolsView,
};
use serde::{Serialize,Deserialize};
use actix_web_dev::error::{
    Result,
};
pub fn pools_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/pools", web::get().to(get_pools_page_info));
}

pub async fn get_pools_page_info(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    Ok(HttpResponse::Ok().json(json!(
        {
            "achiei": PoolsView::get(&conn).await?,
        }
    )))
}

