use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::UserError;

#[derive(Serialize)]
struct MessageResponse {
    message: &'static str,
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/ingest")]
async fn ingest_data(db: web::Data<PgPool>) -> Result<HttpResponse, UserError> {
    // contracts first (foreign-key associations)
    use crate::ingest::ingest_contracts::ingest_contracts;
    ingest_contracts(&db).await.map_err(|e| {
        //
        log::warn!("ingest_data() failed with: {e}");
        UserError::InternalError
    })?;
    // claims
    use crate::ingest::ingest_claims::ingest_claims;
    ingest_claims(&db).await.map_err(|e| {
        log::warn!("ingest_claims() failed with: {e}");
        UserError::InternalError
    })?;
    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "ingestion successfull",
    }))
}

#[post("/claims/{external_id}/decision")]
async fn post_decide_claim(
    db: web::Data<PgPool>,
    external_id: web::Path<u32>,
) -> Result<HttpResponse, UserError> {
    log::warn!("started dicision process");
    let decision = crate::decide::decide_claim(&db, *external_id as i64).await?;
    crate::db::decision::save_decision(&db, decision)
        .await
        .map_err(|e| {
            //
            log::warn!("save_decision() failed with: {e}");
            UserError::DecisionConflict
        })?;

    Ok(HttpResponse::Ok().json(MessageResponse {
        message: "claim decided successfully",
    }))
}
