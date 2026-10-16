use crate::model::{Claim, ClaimState, ClaimType};
use quick_xml::de::from_str;
use serde::Deserialize;
use std::fs;
use std::io;

pub fn ingest_claims() {
    let path = "../data";
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    entries.sort();

    for entry in entries {
        let path = entry.clone().into_string().unwrap();

        if path.contains("claim") {
            let xml = fs::read_to_string(path).unwrap();
            let claims = parse_claims_xml(&xml);
            crate::db::repo::save_claims(claims);
        }
    }
}
fn parse_claims_xml(xml: &str) -> Vec<Claim> {
    //
    let export: Export = from_str(xml).unwrap();

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
                FieldName::State => {
                    dbg!(&field.text);
                    claim_state = text.parse().expect("enum variants");
                }
                FieldName::SaleOrderNumber => sale_order_number = text.to_string(),
                FieldName::IncidentDate => incident_date = text.parse().ok(),
                FieldName::ReportedDate => reported_date = text.parse().ok(),
                FieldName::RepairCost => repair_cost = text.parse().ok(),
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
    claims
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

#[cfg(test)]
mod test {

    #[test]
    fn test_parse_claims_xml() {
        use crate::ingestion::ingest_claims::parse_claims_xml;
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
  </export>
        "#;

        let claims = parse_claims_xml(xml);

        dbg!(claims);
    }

    #[test]
    fn test_ingest_claims() {
        use crate::ingestion::ingest_claims::ingest_claims;

        ingest_claims();
    }
}
