use chrono::Utc;

use crate::{
    error::UserError,
    model::{Claim, ClaimType, Contract, ContractStatus, Decision, DecisionReason, DecisionStatus},
};

use sqlx::PgPool;

use rust_decimal::Decimal;

pub(crate) async fn decide_claim(db: &PgPool, external_id: i64) -> Result<Decision, UserError> {
    // get claim and contract
    let claim_opt = crate::db::claim::get_claim_by_external_id(db, external_id)
        .await
        .map_err(|e| {
            //
            log::warn!("get_claim_by_external_id: failed with: {e}");
            UserError::InternalError
        })?;
    let Some(claim) = claim_opt else {
        return Err(UserError::ClaimNotFound);
    };
    let contract_opt =
        crate::db::contract::get_contract_by_sale_order_number(db, &claim.sale_order_number)
            .await
            .map_err(|e| {
                //
                log::warn!("get_contract_by_sale_order_number: failed with: {e}");
                UserError::InternalError
            })?;
    let Some(contract) = contract_opt else {
        return Ok(Decision {
            id: external_id,
            timestamp: Utc::now(),
            decision_status: DecisionStatus::Rejected,
            payout: None,
            decision_reason: DecisionReason::OrphanClaim,
        });
    };
    // check conditions
    let decision = check_conditions(&claim, &contract);

    Ok(decision)
}

fn check_conditions(claim: &Claim, contract: &Contract) -> Decision {
    if !preconditions_ok(claim, contract) {
        return Decision {
            id: claim.external_id,
            timestamp: Utc::now(),
            decision_status: DecisionStatus::Rejected,
            payout: None,
            decision_reason: DecisionReason::PreconditionsNotMet,
        };
    }
    // check other conditions
    //    ### Per-type rules
    match claim.claim_type {
        //| `THEFT` | police report number present, otherwise `REJECTED` | residual value | — |
        ClaimType::Theft => {
            //return decide_theft(claim, contract);
        }
        //| `PARTIAL_DAMAGE` | `repair_cost` must exceed the €50 deductible, otherwise `REJECTED` | `repair_cost − 50.00` | — |
        ClaimType::PartialDamage => {
            //
        }
        //| `TOTAL_DAMAGE` | — | `residual value − 150.00` (not below 0) | contract state → `done` (ends early) |
        ClaimType::TotalDamage => {
            //
        }
        ClaimType::PaymentDefault => {
            //
        }
        ClaimType::Unknown => {
            //
        } //| Type | Extra requirement | Payout | Side effect |
          //
          //The feed may contain claim types not listed here. Your service must not crash on
          //them; how you handle them is your design decision.
          //
    };

    todo!()
}

fn preconditions_ok(claim: &Claim, contract: &Contract) -> bool {
    // 1. The referenced contract exists.
    if claim.sale_order_number != contract.sale_order_number {
        return false;
    }
    // 2. The contract is in state `active` **at the time the decision is made**.
    if contract.contract_status != ContractStatus::Active {
        return false;
    }
    // 3. The claim's `incident_date` lies within the leasing period
    //   (`start_leasing` ≤ incident ≤ `end_leasing`).
    let (Some(start), Some(end)) = (contract.start_leasing, contract.end_leasing) else {
        return false;
    };
    let Some(incident_date) = claim.incident_date else {
        return false;
    };
    if !(start..=end).contains(&incident_date) {
        return false;
    };
    if !has_fully_contained_months_matching_term(claim, contract) {
        return false;
    }

    true
}

fn has_fully_contained_months_matching_term(claim: &Claim, contract: &Contract) -> bool {
    use chrono::Datelike;
    use chrono::Months;
    let term = contract.term;
    let Some(start) = contract.start_leasing else {
        return false;
    };
    let Some(end) = contract.end_leasing else {
        return false;
    };
    // if start starts with the first and end ends with the last day of the month
    if start.day() == 1 {
        if let Some(start_plus_term) = start.checked_add_months(Months::new(term as u32 - 1)) {
            if start_plus_term == end {
                return true;
            };
        };
        return true;
    }
    // if start is not the first - start with the first of next month

    todo!()
}

//| `THEFT` | police report number present, otherwise `REJECTED` | residual value | — |
//| `PARTIAL_DAMAGE` | `repair_cost` must exceed the €50 deductible, otherwise `REJECTED` | `repair_cost − 50.00` | — |
//| `TOTAL_DAMAGE` | — | `residual value − 150.00` (not below 0) | contract state → `done` (ends early) |

//### Residual value
//residual_value = net_value × remaining_months / total_months
//- `total_months` = calendar months **fully contained** in
//  `[start_leasing, end_leasing]`
//- `remaining_months` = calendar months **fully contained** in
//  `[incident_date, end_leasing]`
//- Round money **half up** to cents, and use an exact decimal representation for
//  money; binary floating-point arithmetic introduces rounding errors.
fn calc_residual_value(claim: &Claim, contract: &Contract) -> Option<Decimal> {
    todo!()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_fully_contained_months() {
        //
        //
        //
    }
}
