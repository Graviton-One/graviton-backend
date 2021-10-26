use actix_web::{
    App, 
    HttpServer, 
    web
};
use api::*;
use actix_cors::Cors;
use api::staking::routes::staking_routes;
use api::pools::routes::pools_routes;
use api::farming::routes::farms_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .data(pool.clone())
            .service(
                web::scope("/apiv2")
                    .configure(staking_routes)
                    .configure(pools_routes)
                    .configure(farms_routes)
            )
    })
    .bind("0.0.0.0:8088")?
    .run()
    .await
}
