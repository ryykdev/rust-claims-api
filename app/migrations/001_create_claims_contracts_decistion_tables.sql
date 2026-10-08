DROP TABLE IF EXISTS decision;
DROP TABLE IF EXISTS claim;
DROP TABLE IF EXISTS contract;

CREATE TABLE IF NOT EXISTS contracts (
    name VARCHAR(255) PRIMARY KEY,
    sale_order_number VARCHAR(255) NOT NULL UNIQUE,
    partner_id INT,
    partner_name VARCHAR(255),
    customer_company_id INT,
    customer_company_name VARCHAR(255),
    frame_number VARCHAR(255) NOT NULL,
    brand VARCHAR(255) NOT NULL,
    model VARCHAR(255) NOT NULL,
    net_value NUMERIC(12, 2),
    start_leasing DATE,
    end_leasing DATE,
    term INT,
    insurance_rate NUMERIC(12, 2),
    status VARCHAR(50) NOT NULL
    );

CREATE TABLE IF NOT EXISTS claims (
    id BIGINT PRIMARY KEY,
    claim_type VARCHAR(255) NOT NULL,
    claim_state VARCHAR(255) NOT NULL,
    sale_order_number VARCHAR(255),
    incident_date DATE,
    reported_date DATE,
    repair_cost NUMERIC(12,2) NOT NULL,
    police_report VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    CONSTRAINT fk_claim_contract
    FOREIGN KEY (sale_order_number)
    REFERENCES contracts (sale_order_number)
    );
CREATE INDEX IF NOT EXISTS idx_claims_sale_order ON claims (sale_order_number);

CREATE TABLE IF NOT EXISTS decisions (
    id BIGINT PRIMARY KEY,
    datetime TIMESTAMP NOT NULL,
    status VARCHAR(255) NOT NULL,
    reason TEXT NOT NULL,
    payout NUMERIC(12,2) NOT NULL,
    CONSTRAINT fk_decision_claim
    FOREIGN KEY (id)
    REFERENCES claims (id)
    );
