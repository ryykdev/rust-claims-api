use core::result;

use actix_web::{Responder, get, post, web};
use sqlx::PgPool;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/ingest")]
async fn ingest_data(db: web::Data<PgPool>) -> impl Responder {
    // contracts first (foreign-key associations)
    use crate::ingest::ingest_contracts::ingest_contracts;
    let result_contracts = ingest_contracts(&db).await;
    if let Err(e) = result_contracts {
        log::warn!("ingest_data:conctracts: failed with: {e}");
        return format!("ingestion error");
    }
    // claims
    use crate::ingest::ingest_claims::ingest_claims;
    let result_claims = ingest_claims(&db).await;
    if let Err(e) = result_claims {
        log::warn!("ingest_data:claims failed with: {e}");
        return format!("ingestion error");
    }
    format!("ingestion ok")
}
