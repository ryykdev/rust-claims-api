mod model;

use actix_web::{App, HttpServer, Responder, get, web};
use sqlx::postgres::PgPoolOptions;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    //
    let db_url = std::env::var("DATABASE_URL").expect("db url in env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    HttpServer::new(|| App::new().service(greet))
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}
