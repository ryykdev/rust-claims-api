DROP TABLE IF EXISTS decision;
DROP TABLE IF EXISTS claim;
DROP TABLE IF EXISTS contract;

create type contract_status as enum (
  'Active',
  'Done', 
  'Cancel', 
  'Unknown'
);


CREATE TABLE IF NOT EXISTS contracts (
    name VARCHAR(255) PRIMARY KEY,
    sale_order_number VARCHAR(255) NOT NULL UNIQUE,
    partner_id INT,
    partner_name VARCHAR(255),
    company_id INT,
    company_name VARCHAR(255),
    frame_number VARCHAR(255) NOT NULL,
    brand VARCHAR(255) NOT NULL,
    model VARCHAR(255) NOT NULL,
    net_value NUMERIC(12, 2),
    start_leasing DATE,
    end_leasing DATE,
    term INT,
    insurance_rate NUMERIC(12, 2),
    contract_status contract_status NOT NULL
    );

create type claim_type as enum (
    'Theft',
    'PartialDamage',
    'TotalDamage',
    'PaymentDefault',
    'Unknown'
);

create type claim_state as enum (
    'Investigation',
    'AwaitingDocuments',
    'Settled',
    'NotSettled',
    'Invalid',
    'Unknown'
);

CREATE TABLE IF NOT EXISTS claims (
    external_id BIGINT PRIMARY KEY,
    claim_type claim_type NOT NULL,
    claim_state claim_state NOT NULL,
    sale_order_number VARCHAR(255) NOT NULL,
    incident_date DATE,
    reported_date DATE,
    repair_cost NUMERIC(12,2),
    police_report_no VARCHAR(255),
    description TEXT NOT NULL
    );

CREATE INDEX IF NOT EXISTS idx_claims_sale_order ON claims (sale_order_number);

create type decision_status as enum (
    'Approved',
    'Rejected',
    'ManualReview',
    'Unknown'
);

create type decision_reason as enum (
    'ContractNotActive',
    'OrphanClaim',
    'ImplausibleDate',
    'IncidentOutsideLease',
    'ClaimStateFinal',
    'MissingPoliceReport',
    'NotFullyContainedMonths',
    'RepairCostInsideDeductible',
    'MissingRepairCost',
    'PaymentDefault',
    'UnknownClaimType',
    'MultipleContractEndingClaims',
    'AfterContractEndingClaim',
    'Unknown'
);

CREATE TABLE IF NOT EXISTS decisions (
    id BIGINT PRIMARY KEY,
    datetime TIMESTAMP NOT NULL,
    decision_status decision_status NOT NULL,
    decision_reason decision_reason NOT NULL,
    payout NUMERIC(12,2) NOT NULL,
    CONSTRAINT fk_decision_claim
    FOREIGN KEY (id)
    REFERENCES claims (external_id)
    );
