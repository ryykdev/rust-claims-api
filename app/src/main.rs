mod db;
mod error;
mod ingest;
mod model;
mod routes;

use routes::{greet, ingest_data};

use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("starting HTTP server at http://localhost:8000");

    let db_url = std::env::var("DATABASE_URL").expect("db url in env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(greet)
            .service(ingest_data)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
