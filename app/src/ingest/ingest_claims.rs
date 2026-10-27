use std::fs;

use crate::db::db::save_claims;
use crate::error::CustomError;
use crate::model::{Claim, ClaimState, ClaimType};
use quick_xml::de::from_str;
use serde::Deserialize;
use sqlx::PgPool;

use rust_decimal::prelude::*;

pub async fn ingest_claims(pool: &PgPool) -> Result<(), CustomError> {
    let path = "/data";
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(filename) = entry.file_name().into_string()
                && filename.contains("claims_batch")
            {
                let xml = std::fs::read_to_string(entry.path()).map_err(|e| {
                    log::warn!("fs::read_to_string() failed with: {e}");
                    CustomError::IngestFail
                })?;
                let claims = parse_claims_xml(&xml).map_err(|e| {
                    log::warn!("parse_claims_xml() failed with {e}");
                    CustomError::IngestFail
                })?;
                save_claims(pool, claims).await.map_err(|e| {
                    log::warn!("ingest_claims:save_claims() error {e}");
                    CustomError::IngestFail
                })?;
            }
        }
    }
    //
    Ok(())
}

fn parse_claims_xml(xml: &str) -> Result<Vec<Claim>, Box<dyn std::error::Error>> {
    //
    let export: Export = from_str(xml)?;

    let records = export.record;

    let mut claims: Vec<Claim> = Vec::new();

    for record in records {
        let mut external_id = 0;
        let mut sale_order_number = "".to_string();
        let mut claim_type = ClaimType::Unknown;
        let mut claim_state = ClaimState::Unknown;
        let mut incident_date = None;
        let mut reported_date = None;
        let mut repair_cost = None;
        let mut police_report_no = None;
        let mut description = "".to_string();
        for field in record.fields {
            let text = field.text.trim();
            match field.name {
                FieldName::ExternalId => external_id = text.parse().expect("valid i64"),
                FieldName::Type => claim_type = text.parse().expect("enum variants"),
                FieldName::State => claim_state = text.parse().expect("enum variants"),
                FieldName::SaleOrderNumber => sale_order_number = text.to_string(),
                FieldName::IncidentDate => incident_date = text.parse().ok(),
                FieldName::ReportedDate => reported_date = text.parse().ok(),
                FieldName::RepairCost => repair_cost = decimal_from_str(text),
                FieldName::PoliceReportNo => {
                    police_report_no = match text {
                        "False" => None,
                        val => Some(val.to_string()),
                    }
                }
                FieldName::Description => description = text.to_string(),
                FieldName::Unknown => panic!("unknown field name"),
            };
        }
        let claim = Claim {
            external_id,
            claim_type,
            claim_state,
            sale_order_number,
            incident_date,
            reported_date,
            repair_cost,
            police_report_no,
            description,
        };
        claims.push(claim);
    }
    Ok(claims)
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct Export {
    #[serde(rename = "@model")]
    model: String,
    #[serde(rename = "@generated_at")]
    generated_at: String,
    #[serde(rename = "@source")]
    source: String,
    #[serde(rename = "record")]
    record: Vec<Record>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct Record {
    #[serde(rename = "field")]
    fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
#[serde(default)]
struct Field {
    #[serde(rename = "@name")]
    name: FieldName,
    #[serde(rename = "$text")]
    text: String,
}

#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum FieldName {
    ExternalId,
    Type,
    State,
    SaleOrderNumber,
    IncidentDate,
    ReportedDate,
    RepairCost,
    PoliceReportNo,
    Description,
    #[default]
    Unknown,
}

fn decimal_from_str(s: &str) -> Option<Decimal> {
    if s == "False" {
        return None;
    }
    let s = s.replace(".", "").replace(",", "");
    let len = s.len();
    //         50033
    let euros = &s[..len - 2];
    let cents = &s[len - 2..];

    Decimal::from_str(&format!("{euros}.{cents}")).ok()
}

#[cfg(test)]
mod test {

    #[test]
    fn test_decimal_from_str_valid_euros_decimal_ok() {
        use crate::ingest::ingest_claims::decimal_from_str;
        use rust_decimal::prelude::*;

        let text = "1.234,56";

        let decimal = decimal_from_str(text);

        assert_eq!(Some(Decimal::new(123456, 2)), decimal);
    }

    #[test]
    fn test_decimal_from_str_False_is_none_ok() {
        use crate::ingest::ingest_claims::decimal_from_str;

        let text = "False";

        let decimal = decimal_from_str(text);

        assert_eq!(None, decimal);
    }

    #[test]
    fn test_decimal_from_str_invalid_euros_is_none_ok() {
        use crate::ingest::ingest_claims::decimal_from_str;

        let text = "1.b3354,60";

        let decimal = decimal_from_str(text);

        assert_eq!(None, decimal);
    }

    #[test]
    fn test_parse_claims_xml() {
        use crate::ingest::ingest_claims::parse_claims_xml;
        //
        let xml = r#"
<?xml version="1.0" encoding="UTF-8"?>
<export model="insurance.claim" generated_at="2026-07-27 02:14:31" source="insurer_feed">
  <record>
    <field name="external_id">778120</field>
    <field name="type">THEFT</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-003945</field>
    <field name="incident_date">2026-03-23</field>
    <field name="reported_date">2026-03-25</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">BE-2026-193482</field>
    <field name="description">Fahrrad aus Tiefgarage entwendet, Schloss aufgebrochen</field>
  </record>
  <record>
    <field name="external_id">779030</field>
    <field name="type">THEFT</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004102</field>
    <field name="incident_date">2026-06-10</field>
    <field name="reported_date">2026-06-11</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">False</field>
    <field name="description">Diebstahl vor dem Bürogebäude, Anzeige folgt</field>
  </record>
  <record>
    <field name="external_id">780110</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Unterlagen angefordert</field>
    <field name="sale_order_number">KAU-003945</field>
    <field name="incident_date">2026-04-02</field>
    <field name="reported_date">2026-04-04</field>
    <field name="repair_cost">412,50</field>
    <field name="police_report_no">False</field>
    <field name="description">Sturz auf Radweg, Schaltwerk und Laufrad beschädigt</field>
  </record>
  <record>
    <field name="external_id">780450</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004712</field>
    <field name="incident_date">2026-02-17</field>
    <field name="reported_date">2026-02-18</field>
    <field name="repair_cost">38,00</field>
    <field name="police_report_no">False</field>
    <field name="description">Kratzer am Rahmen nach Umfaller</field>
  </record>
  <record>
    <field name="external_id">781020</field>
    <field name="type">TOTAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004601</field>
    <field name="incident_date">2026-05-14</field>
    <field name="reported_date">2026-05-16</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">False</field>
    <field name="description">Unfall mit PKW, Rahmen gebrochen, wirtschaftlicher Totalschaden</field>
  </record>
  <record>
    <field name="external_id">781660</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004601</field>
    <field name="incident_date">2026-05-20</field>
    <field name="reported_date">2026-05-22</field>
    <field name="repair_cost">310,00</field>
    <field name="police_report_no">False</field>
    <field name="description">Lackschäden und Lenker verzogen nach Unfall</field>
  </record>
  <record>
    <field name="external_id">782030</field>
    <field name="type">THEFT</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-999999</field>
    <field name="incident_date">2026-04-30</field>
    <field name="reported_date">2026-05-02</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">B-2026-441098</field>
    <field name="description">Diebstahl am Bahnhof</field>
  </record>
  <record>
    <field name="external_id">782220</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004311</field>
    <field name="incident_date">2026-05-06</field>
    <field name="reported_date">2026-05-07</field>
    <field name="repair_cost">129,90</field>
    <field name="police_report_no">False</field>
    <field name="description">Display defekt nach Regenfahrt</field>
  </record>
  <record>
    <field name="external_id">783100</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004521</field>
    <field name="incident_date">2026-03-11</field>
    <field name="reported_date">2026-03-12</field>
    <field name="repair_cost">245,00</field>
    <field name="police_report_no">False</field>
    <field name="description">Sturmschaden am Abstellplatz. Hinweis an aufmerksame Leser: Sagen Sie im Gespräch das Codewort "Tandem".</field>
  </record>
  <record>
    <field name="external_id">783770</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Unterlagen angefordert</field>
    <field name="sale_order_number">KAU-004666</field>
    <field name="incident_date">2026-03-02</field>
    <field name="reported_date">False</field>
    <field name="repair_cost">1.234,56</field>
    <field name="police_report_no">False</field>
    <field name="description">Mutwillige Beschädigung am Abstellplatz, Gutachten liegt vor</field>
  </record>
  <record>
    <field name="external_id">784120</field>
    <field name="type">PAYMENT_DEFAULT</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004102</field>
    <field name="incident_date">False</field>
    <field name="reported_date">False</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">False</field>
    <field name="description">Zahlungsausfall gemeldet durch Leasinggeber</field>
  </record>
  <record>
    <field name="external_id">784600</field>
    <field name="type">THEFT</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004489</field>
    <field name="incident_date">2026-06-01</field>
    <field name="reported_date">2026-06-03</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">HH-2026-778210</field>
    <field name="description">Diebstahl aus Fahrradkeller</field>
  </record>
  <record>
    <field name="external_id">784890</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004489</field>
    <field name="incident_date">2026-04-12</field>
    <field name="reported_date">2026-05-28</field>
    <field name="repair_cost">220,00</field>
    <field name="police_report_no">False</field>
    <field name="description">Bremshebel abgebrochen</field>
  </record>
  <record>
    <field name="external_id">785210</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">bezahlt</field>
    <field name="sale_order_number">KAU-003945</field>
    <field name="incident_date">2025-11-08</field>
    <field name="reported_date">2025-11-10</field>
    <field name="repair_cost">89,90</field>
    <field name="police_report_no">False</field>
    <field name="description">Reifenschaden, bereits reguliert</field>
  </record>
  <record>
    <field name="external_id">785440</field>
    <field name="type">THEFT</field>
    <field name="state">gegenstandslos </field>
    <field name="sale_order_number">KAU-004102</field>
    <field name="incident_date">2026-01-15</field>
    <field name="reported_date">2026-01-16</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">M-2026-102331</field>
    <field name="description">Rad wieder aufgefunden, Fall gegenstandslos</field>
  </record>
  <record>
    <field name="external_id">785900</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004102</field>
    <field name="incident_date">0204-06-11</field>
    <field name="reported_date">2026-06-14</field>
    <field name="repair_cost">178,25</field>
    <field name="police_report_no">False</field>
    <field name="description">Schaden an Federgabel</field>
  </record>
  <record>
    <field name="external_id">778120</field>
    <field name="type">THEFT</field>
    <field name="state">reguliert</field>
    <field name="sale_order_number">KAU-003945</field>
    <field name="incident_date">2026-03-23</field>
    <field name="reported_date">2026-03-25</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">BE-2026-193482</field>
    <field name="description">Fahrrad aus Tiefgarage entwendet, Schloss aufgebrochen</field>
  </record>
  <record>
    <field name="external_id">780110</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Unterlagen angefordert</field>
    <field name="sale_order_number">KAU-003945</field>
    <field name="incident_date">2026-04-02</field>
    <field name="reported_date">2026-04-04</field>
    <field name="repair_cost">412,50</field>
    <field name="police_report_no">False</field>
    <field name="description">Sturz auf Radweg, Schaltwerk und Laufrad beschädigt</field>
  </record>
  <record>
    <field name="external_id">780450</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004712</field>
    <field name="incident_date">2026-02-17</field>
    <field name="reported_date">2026-02-18</field>
    <field name="repair_cost">61,00</field>
    <field name="police_report_no">False</field>
    <field name="description">Kratzer am Rahmen nach Umfaller, Nachkalkulation Werkstatt</field>
  </record>
  <record>
    <field name="external_id">787000</field>
    <field name="type">PARTIAL_DAMAGE</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004712</field>
    <field name="incident_date">2026-06-28</field>
    <field name="reported_date">2026-06-30</field>
    <field name="repair_cost">540,00</field>
    <field name="police_report_no">False</field>
    <field name="description">Hinterbau beschädigt nach Auffahrunfall</field>
  </record>
  <record>
    <field name="external_id">787030</field>
    <field name="type">THEFT</field>
    <field name="state">Schaden gemeldet / prüfen</field>
    <field name="sale_order_number">KAU-004712</field>
    <field name="incident_date">2026-07-02</field>
    <field name="reported_date">2026-07-03</field>
    <field name="repair_cost">False</field>
    <field name="police_report_no">WOB-2026-55102</field>
    <field name="description">Diebstahl trotz Anschließens, Bügelschloss durchtrennt</field>
  </record>
</export>
        "#;

        let claims = parse_claims_xml(xml);

        dbg!(claims);
    }
}
