use std::str::FromStr;
use std::time::SystemTime;
use strum_macros::EnumString;

use chrono::NaiveDate;
use rust_decimal::Decimal;

use sqlx::Type;

#[derive(sqlx::FromRow, Debug)]
pub struct Claim {
    pub external_id: i64,
    pub claim_type: ClaimType,
    pub claim_state: ClaimState,
    pub sale_order_number: String, // foreign key from contract - orphan claims possible
    pub incident_date: Option<NaiveDate>, // claim requires an incident date
    pub reported_date: Option<NaiveDate>,
    pub repair_cost: Option<Decimal>,
    pub police_report_no: Option<String>,
    pub description: String,
}

#[derive(Debug, PartialEq, EnumString, Type)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "claim_type", rename_all = "PascalCase")]
pub enum ClaimType {
    Theft,
    PartialDamage,
    TotalDamage,
    PaymentDefault,
    Unknown,
}

impl From<String> for ClaimType {
    fn from(s: String) -> Self {
        ClaimType::from_str(&s).unwrap_or(ClaimType::Unknown)
    }
}

#[derive(Debug, PartialEq, EnumString, Type)]
#[sqlx(type_name = "claim_state", rename_all = "PascalCase")]
pub enum ClaimState {
    #[strum(serialize = "Schaden gemeldet / prüfen")]
    Investigation,
    #[strum(serialize = "Unterlagen angefordert")]
    AwaitingDocuments,
    #[strum(serialize = "reguliert", serialize = "bezahlt")]
    Settled,
    #[strum(serialize = "abgelehnt")]
    NotSettled,
    #[strum(serialize = "gegenstandslos")]
    Invalid,
    Unknown,
}

impl From<String> for ClaimState {
    fn from(s: String) -> Self {
        ClaimState::from_str(&s).unwrap_or(ClaimState::Unknown)
    }
}

#[derive(sqlx::FromRow, Debug, Type)]
pub struct Contract {
    pub name: String,
    pub sale_order_number: String, //
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
    pub term: i32,
    pub insurance_rate: Decimal,
    pub contract_status: ContractStatus,
}

#[derive(Debug, PartialEq, EnumString, Type)]
#[sqlx(type_name = "contract_status", rename_all = "PascalCase")]
pub enum ContractStatus {
    #[strum(ascii_case_insensitive)]
    Active,
    #[strum(ascii_case_insensitive)]
    Done,
    #[strum(ascii_case_insensitive)]
    Cancel,
    Unknown,
}

impl From<String> for ContractStatus {
    fn from(s: String) -> Self {
        ContractStatus::from_str(&s).unwrap_or(ContractStatus::Unknown)
    }
}

#[derive(sqlx::FromRow)]
pub struct Decision {
    pub claim: i64, // claim.external_id
    pub timestamp: SystemTime,
    pub decision_status: DecisionStatus,
    pub payout: Decimal,
    pub decision_reason: DecisionReason,
}

#[derive(Debug, PartialEq, EnumString, Type)]
pub enum DecisionStatus {
    Approved,
    Rejected,
    ManualReview,
    Unknown,
}

impl From<String> for DecisionStatus {
    fn from(s: String) -> Self {
        DecisionStatus::from_str(&s).unwrap_or(DecisionStatus::Unknown)
    }
}

#[derive(Debug, PartialEq, EnumString, Type)]
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
    Unknown,
}

impl From<String> for DecisionReason {
    fn from(s: String) -> Self {
        DecisionReason::from_str(&s).unwrap_or(DecisionReason::Unknown)
    }
}
