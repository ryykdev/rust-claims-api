use std::time::SystemTime;

use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(sqlx::FromRow)]
pub struct Claim {
    pub external_id: i64,
    pub claim_type: ClaimType,
    pub claim_state: ClaimState,
    pub sale_order_number: String,        // foreign key from contract
    pub incident_date: Option<NaiveDate>, // claim requires an incident date
    pub reported_date: Option<NaiveDate>,
    pub repair_cost: Decimal,
    pub police_report: Option<String>,
    pub description: Option<String>,
}

pub enum ClaimType {
    Theft,
    PartialDamage,
    TotalDamage,
    PaymentDefault,
    Unknown,
}

pub enum ClaimState {
    Investigation,
    AwaitingDocuments,
    Settled,
    NotSettled,
    Invalid,
    Unknown,
}

#[derive(sqlx::FromRow)]
pub struct Contract {
    //
    pub name: String,
    pub sale_order_number: String,
    pub partner_id: i64,
    pub partner_name: String,
    pub customer_company_id: i64,
    pub customer_company_name: String,
    pub frame_number: i64,
    pub brand: String,
    pub model: String,
    pub net_value: Decimal,
    pub start_leasing: NaiveDate,
    pub end_leasing: NaiveDate,
    pub term: i64,
    pub insurance_rate: Decimal,
    pub contract_status: ContractStatus,
}

pub enum ContractStatus {
    Active,
    Done,
    Canceled,
    Unknown,
}

#[derive(sqlx::FromRow)]
pub struct Decision {
    pub claim: i64, // claim.external_id
    pub timestamp: SystemTime,
    pub status: DecisionStatus,
    pub payout: Decimal,
    pub reason: DecisionReason,
}

pub enum DecisionStatus {
    Approved,
    Rejected,
    ManualReview,
}

pub enum DecisionReason {
    ContractNotActive,
    OrphanClaim,
    ImplausibleDate,
    IncidentOutsideLease,
    ClaimStateFinal,
    MissingPoliceReport,
    NotFullyContainedMonths,
    RepairCostInsideDeductible,
    MissingRepairCost,
    PaymentDefault,
    UnknownClaimType,
    MultipleContractEndingClaims,
    AfterContractEndingClaim,
}
