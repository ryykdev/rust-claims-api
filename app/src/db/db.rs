use crate::model::{Claim, ClaimState, ClaimType, Contract, ContractStatus};
use sqlx::PgPool;

pub async fn save_claims(
    pool: &PgPool,
    claims: Vec<Claim>,
) -> Result<(), Box<dyn std::error::Error>> {
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
            Ok(_) => {}
            Err(e) => {
                log::warn!("save_claims() with error: {e}");
            }
        };
    }

    Ok(())
}

pub async fn save_contracts(
    pool: &PgPool,
    contracts: Vec<Contract>,
) -> Result<(), Box<dyn std::error::Error>> {
    //
    for contract in contracts {
        let result = sqlx::query_as!(
            Contract,
            r#"
    INSERT INTO contracts (
    name,
    sale_order_number,
    partner_id,
    partner_name,
    company_id,
    company_name,
    frame_number,
    brand,
    model,
    net_value,
    start_leasing,
    end_leasing,
    term,
    insurance_rate,
    contract_status
    )

    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            contract.name,
            contract.sale_order_number,
            contract.partner_id,
            contract.partner_name,
            contract.company_id,
            contract.company_name,
            contract.frame_number,
            contract.brand,
            contract.model,
            contract.net_value,
            contract.start_leasing,
            contract.end_leasing,
            contract.term,
            contract.insurance_rate,
            contract.contract_status as ContractStatus
        )
        .execute(pool)
        .await;

        match result {
            Ok(_) => {}
            Err(e) => {
                log::warn!("save_contracts() with error: {e}");
            }
        };
    }

    Ok(())
}
