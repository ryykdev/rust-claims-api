use core::error::Error;

use crate::model::{Contract, ContractStatus};
use sqlx::PgPool;

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

pub async fn get_contract_by_sale_order_number(
    pool: &PgPool,
    sale_order_number: &String,
) -> Result<Option<Contract>, Box<dyn Error>> {
    //
    let contract: Option<Contract> = sqlx::query_as!(
        Contract,
        r#"
    SELECT 
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
    contract_status as "contract_status!: ContractStatus"
    FROM contracts WHERE sale_order_number = $1
    "#,
        sale_order_number
    )
    .fetch_optional(pool)
    .await?;

    Ok(contract)
}
