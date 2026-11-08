use chrono::Utc;

use crate::{
    error::UserError,
    model::{Claim, ClaimType, Contract, ContractStatus, Decision, DecisionReason, DecisionStatus},
};

use sqlx::PgPool;

use rust_decimal::Decimal;

pub(crate) async fn decide_claim(db: &PgPool, external_id: i64) -> Result<Decision, UserError> {
    // get claim, contract and associated_claims of same contract
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
    let associated_claims =
        crate::db::claim::get_associated_claims_for_sale_order_number(db, &claim.sale_order_number)
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
    let decision = check_conditions(&claim, &contract, &associated_claims)?;

    Ok(decision)
}

fn check_conditions(
    claim: &Claim,
    contract: &Contract,
    associated_claims: &[Claim],
) -> Result<Decision, UserError> {
    if !preconditions_ok(claim, contract) {
        return Ok(Decision {
            id: claim.external_id,
            timestamp: Utc::now(),
            decision_status: DecisionStatus::Rejected,
            payout: None,
            decision_reason: DecisionReason::PreconditionsNotMet,
        });
    }
    // if claim is not in decidable state return Conflict 409
    if !claim.is_decidable() {
        return Err(UserError::DecisionConflict);
    }
    // check other conditions
    //    ### Per-type rules
    match claim.claim_type {
        //| `THEFT` | police report number present, otherwise `REJECTED` | residual value | — |
        ClaimType::Theft => {
            //return decide_theft(claim, contract, associated_claims);
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

    true
}

//| `THEFT` | police report number present, otherwise `REJECTED` | residual value | — |
fn decide_theft(claim: &Claim, contract: &Contract, associated_claims: &[Claim]) -> Decision {
    if claim.police_report_no.is_none() {
        return Decision {
            id: claim.external_id,
            timestamp: Utc::now(),
            decision_status: DecisionStatus::Rejected,
            payout: None,
            decision_reason: DecisionReason::MissingPoliceReport,
        };
    }
    if !has_fully_contained_months_matching_term(contract) {
        return Decision {
            id: claim.external_id,
            timestamp: Utc::now(),
            decision_status: DecisionStatus::Rejected,
            payout: None,
            decision_reason: DecisionReason::NotFullyContainedMonths,
        };
    }
    let residual_value = calc_residual_value(claim, contract);
    todo!()
}
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

// function guards against terms bigger than ten years with false
fn has_fully_contained_months_matching_term(contract: &Contract) -> bool {
    use chrono::Datelike;
    use chrono::Days;
    use chrono::Months;
    let term = contract.term;
    let ten_year_term = 12 * 10;
    // guard against an implausible term
    if term > ten_year_term {
        return false;
    }
    let Some(start) = contract.start_leasing else {
        return false;
    };
    let Some(end) = contract.end_leasing else {
        return false;
    };
    // if start starts with the first and end ends with the last day of the month
    if start.day() == 1 {
        let start_plus_term_minus_one_day = start
            .checked_add_months(Months::new(term as u32))
            // we can unwrap because we guarded against implausible terms
            .unwrap()
            .checked_sub_days(Days::new(1))
            .unwrap();
        start_plus_term_minus_one_day == end
    }
    // else term starts with next month
    else {
        //
        let day_of_month = start.day() as u64;
        let start_plus_term_minus_one_day = start
            .checked_add_months(Months::new(term as u32 + 1))
            .unwrap()
            .checked_sub_days(Days::new(day_of_month))
            .unwrap();
        start_plus_term_minus_one_day == end
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        model::{Contract, ContractStatus},
        services::decide::has_fully_contained_months_matching_term,
    };
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    fn contract_active_term_valid_starts_with_first_of_month() -> Contract {
        Contract {
            brand: "SomeBrand".to_string(),
            name: "SomeBike".to_string(),
            sale_order_number: "KAU-123123".to_string(),
            partner_id: 123123,
            partner_name: "Peter Parker".to_string(),
            company_id: 12312,
            company_name: "SomeCompany".to_string(),
            frame_number: "X123Z".to_string(),
            model: "SomeModel".to_string(),
            net_value: Decimal::new(349900, 2),
            start_leasing: Some(NaiveDate::from_ymd_opt(2024, 9, 1).unwrap()),
            end_leasing: Some(NaiveDate::from_ymd_opt(2027, 8, 31).unwrap()),
            term: 36,
            insurance_rate: Decimal::new(453, 2),
            contract_status: ContractStatus::Active,
        }
    }
    fn contract_active_term_valid_starts_with_next_months_first() -> Contract {
        Contract {
            brand: "SomeBrand".to_string(),
            name: "SomeBike".to_string(),
            sale_order_number: "KAU-123123".to_string(),
            partner_id: 123123,
            partner_name: "Peter Parker".to_string(),
            company_id: 12312,
            company_name: "SomeCompany".to_string(),
            frame_number: "X123Z".to_string(),
            model: "SomeModel".to_string(),
            net_value: Decimal::new(349900, 2),
            start_leasing: Some(NaiveDate::from_ymd_opt(2024, 9, 5).unwrap()),
            end_leasing: Some(NaiveDate::from_ymd_opt(2027, 9, 30).unwrap()),
            term: 36,
            insurance_rate: Decimal::new(453, 2),
            contract_status: ContractStatus::Active,
        }
    }

    fn contract_active_term_different_than_lease_duration_returns_false() -> Contract {
        Contract {
            brand: "SomeBrand".to_string(),
            name: "SomeBike".to_string(),
            sale_order_number: "KAU-123123".to_string(),
            partner_id: 123123,
            partner_name: "Peter Parker".to_string(),
            company_id: 12312,
            company_name: "SomeCompany".to_string(),
            frame_number: "X123Z".to_string(),
            model: "SomeModel".to_string(),
            net_value: Decimal::new(349900, 2),
            start_leasing: Some(NaiveDate::from_ymd_opt(2024, 9, 5).unwrap()),
            end_leasing: Some(NaiveDate::from_ymd_opt(2027, 8, 31).unwrap()),
            term: 36,
            insurance_rate: Decimal::new(453, 2),
            contract_status: ContractStatus::Active,
        }
    }

    fn contract_active_end_month_one_over_false() -> Contract {
        Contract {
            brand: "SomeBrand".to_string(),
            name: "SomeBike".to_string(),
            sale_order_number: "KAU-123123".to_string(),
            partner_id: 123123,
            partner_name: "Peter Parker".to_string(),
            company_id: 12312,
            company_name: "SomeCompany".to_string(),
            frame_number: "X123Z".to_string(),
            model: "SomeModel".to_string(),
            net_value: Decimal::new(349900, 2),
            start_leasing: Some(NaiveDate::from_ymd_opt(2024, 9, 5).unwrap()),
            end_leasing: Some(NaiveDate::from_ymd_opt(2027, 10, 31).unwrap()),
            term: 36,
            insurance_rate: Decimal::new(453, 2),
            contract_status: ContractStatus::Active,
        }
    }

    #[test]
    fn test_has_fully_contained_months_matching_term_ok() {
        //
        let contract = contract_active_term_valid_starts_with_first_of_month();

        let res = has_fully_contained_months_matching_term(&contract);
        assert!(res);
    }
    #[test]
    fn test_has_fully_contained_months_matching_term_starts_with_next_month_ok() {
        //
        let contract = contract_active_term_valid_starts_with_next_months_first();

        let res = has_fully_contained_months_matching_term(&contract);
        assert!(res);
    }
    #[test]
    fn test_has_fully_contained_months_matching_term_end_month_is_short_return_false() {
        //
        let contract = contract_active_term_different_than_lease_duration_returns_false();

        let res = has_fully_contained_months_matching_term(&contract);
        assert!(!res);
    }
    #[test]
    fn test_has_fully_contained_months_matching_term_end_month_one_over_return_false() {
        //
        let contract = contract_active_end_month_one_over_false();

        let res = has_fully_contained_months_matching_term(&contract);
        assert!(!res);
    }
}
