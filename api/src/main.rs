use actix_web::{
    App, 
    HttpServer, 
    web
};
use diesel::{
    r2d2::ConnectionManager,
    PgConnection,
};
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .data(pool.clone())
            .service(
                web::scope("/api")
            )
    })
    .bind("0.0.0.0:8088")?
    .run()
    .await
}
