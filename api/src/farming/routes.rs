use crate::DbPool;
use actix_web::{HttpResponse, web::{self, Query}};
use super::db::{
    FarmsView,
};
use serde::{Serialize,Deserialize};
use actix_web_dev::error::{
    Result,
};
pub fn farms_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/farms", web::get().to(get_farms_page_info));
}

pub async fn get_farms_page_info(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let conn = pool.get()?;
    Ok(HttpResponse::Ok().json(json!(
        {
            "farms": FarmsView::get(&conn).await?,
            "tvl_by_dex": FarmsView::totals_by_dex(&conn).await?,
            "tvl_by_chain": FarmsView::totals_by_chain(&conn).await?
        }
    )))
}



