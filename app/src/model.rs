use std::time::SystemTime;
use strum_macros::EnumString;

use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(sqlx::FromRow, Debug)]
pub struct Claim {
    pub external_id: i64,
    pub claim_type: ClaimType,
    pub claim_state: ClaimState,
    pub sale_order_number: String,        // foreign key from contract
    pub incident_date: Option<NaiveDate>, // claim requires an incident date
    pub reported_date: Option<NaiveDate>,
    pub repair_cost: Option<Decimal>,
    pub police_report_no: Option<String>,
    pub description: String,
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ClaimType {
    Theft,
    PartialDamage,
    TotalDamage,
    PaymentDefault,
    Unknown,
}

#[derive(Debug, PartialEq, EnumString)]
pub enum ClaimState {
    #[strum(to_string = "Schaden gemeldet / prüfen")]
    Investigation,
    #[strum(to_string = "Unterlagen angefordert")]
    AwaitingDocuments,
    #[strum(to_string = "reguliert", serialize = "bezahlt")]
    Settled,
    #[strum(to_string = "abgelehnt")]
    NotSettled,
    #[strum(to_string = "gegenstandslos")]
    Invalid,
    Unknown,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Contract {
    //
    pub name: String,
    pub sale_order_number: String,
    pub partner_id: i32,
    pub partner_name: String,
    pub company_id: i32,
    pub company_name: String,
    pub frame_number: String,
    pub brand: String,
    pub model: String,
    pub net_value: Decimal,
    pub start_leasing: NaiveDate,
    pub end_leasing: Option<NaiveDate>,
    pub term: u8,
    pub insurance_rate: Decimal,
    pub contract_status: ContractStatus,
}

#[derive(Debug, PartialEq, EnumString)]
pub enum ContractStatus {
    #[strum(ascii_case_insensitive)]
    Active,
    #[strum(ascii_case_insensitive)]
    Done,
    #[strum(ascii_case_insensitive)]
    Cancel,
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
