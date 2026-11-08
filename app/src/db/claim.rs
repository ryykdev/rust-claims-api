use crate::model::{Claim, ClaimState, ClaimType};
use core::error::Error;
use sqlx::PgPool;

pub async fn save_claims(pool: &PgPool, claims: Vec<Claim>) -> Result<(), Box<dyn Error>> {
    //
    for claim in claims {
        let result = sqlx::query_as!(
            Claim,
            r#"
    INSERT INTO claims (
    external_id,
    claim_type,
    claim_state,
    sale_order_number,
    incident_date,
    reported_date,
    repair_cost,
    police_report_no, 
    description
    ) 
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ON CONFLICT (external_id)
    DO UPDATE SET  
    claim_type = EXCLUDED.claim_type,
    claim_state = EXCLUDED.claim_state,
    sale_order_number = EXCLUDED.sale_order_number,
    incident_date =  EXCLUDED.incident_date,
    reported_date = EXCLUDED.reported_date,
    repair_cost = EXCLUDED.repair_cost,
    police_report_no = EXCLUDED.police_report_no, 
    description = EXCLUDED.description
            "#,
            claim.external_id,
            claim.claim_type as ClaimType,
            claim.claim_state as ClaimState,
            claim.sale_order_number,
            claim.incident_date,
            claim.reported_date,
            claim.repair_cost,
            claim.police_report_no,
            claim.description
        )
        .execute(pool)
        .await;

        match result {
            Ok(_) => {} // continue iter
            Err(e) => {
                log::warn!("save_claims() with error: {e}");
            }
        };
    }

    Ok(())
}

pub(crate) async fn get_claim_by_external_id(
    db: &PgPool,
    external_id: i64,
) -> Result<Option<Claim>, Box<dyn Error>> {
    //
    let claim: Option<Claim> = sqlx::query_as!(
        Claim,
        r#"
    SELECT
    external_id,
    claim_type AS "claim_type!: ClaimType",
    claim_state AS "claim_state!: ClaimState",
    sale_order_number,
    incident_date, 
    reported_date,
    repair_cost,
    police_report_no,
    description
    FROM claims WHERE external_id = $1
        "#,
        external_id
    )
    .fetch_optional(db)
    .await?;

    Ok(claim)
}
