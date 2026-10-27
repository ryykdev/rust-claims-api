use crate::error::CustomError;
use crate::model::Contract;
use crate::model::ContractStatus;
use chrono::NaiveDate;
use quick_xml::de::from_str;
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;

pub async fn ingest_contracts(db: &PgPool) -> Result<(), CustomError> {
    let path = "/data/contracts_export.xml";
    let bytes = std::fs::read(path).expect("contracts xml file");
    let xml = contracts_from_iso_bytes_to_utf8(&bytes);
    let contracts = parse_contracts_xml(&xml).await.map_err(|e| {
        log::warn!("parse_contracts_xml() failed with {e}");
        CustomError::IngestFail
    })?;
    crate::db::db::save_contracts(db, contracts)
        .await
        .map_err(|e| {
            log::warn!("parse_contracts_xml() failed with {e}");
            CustomError::IngestFail
        })?;
    Ok(())
}

async fn parse_contracts_xml(xml: &str) -> Result<Vec<Contract>, CustomError> {
    //
    let export: Export = from_str(xml).unwrap();

    let records = export.record;

    let mut contracts: Vec<Contract> = Vec::new();

    for record in records {
        let mut name = String::new();
        let mut sale_order_number = String::new();
        let mut partner_id_and_name = String::new();
        let mut customer_company_id_and_name = String::new();
        let mut frame_number = String::new();
        let mut brand = String::new();
        let mut model = String::new();
        let mut net_value = Decimal::default();
        let mut start_leasing = NaiveDate::default();
        let mut end_leasing = None;
        let mut insurance_rate = Decimal::default();
        let mut term = 0;
        let mut contract_status = ContractStatus::Unknown;
        for field in record.fields {
            match field.name {
                FieldName::Name => name = field.text.to_string(),
                FieldName::SaleOrderNumber => sale_order_number = field.text.to_string(),
                FieldName::PartnerId => partner_id_and_name = field.text.to_string(),
                FieldName::CustomerCompanyId => {
                    customer_company_id_and_name = field.text.to_string()
                }
                FieldName::FrameNumber => frame_number = field.text.to_string(),
                FieldName::Brand => brand = field.text.to_string(),
                FieldName::Model => model = field.text.to_string(),
                FieldName::Category => net_value = field.text.parse().expect("decimal number"),
                FieldName::NetValue => start_leasing = field.text.parse().expect("start leasing"),
                FieldName::StartLeasing => end_leasing = field.text.parse().ok(),
                FieldName::EndLeasing => term = field.text.parse().expect("term integer"),
                FieldName::Term => {
                    insurance_rate = field.text.parse().expect("insurance rate decimal")
                }
                FieldName::InsuranceRate => {
                    contract_status = field.text.parse().expect("contract_status enum")
                }
                FieldName::Unknown => {}
            };
        }
        // parse partner and company
        let (partner_id, partner_name) = parse_composite_id(&partner_id_and_name);
        let (company_id, company_name) = parse_composite_id(&customer_company_id_and_name);
        let contract = Contract {
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
            contract_status,
        };
        contracts.push(contract);
    }
    Ok(contracts)
}

fn contracts_from_iso_bytes_to_utf8(bytes: &[u8]) -> String {
    use encoding_rs::Encoding;
    //
    let encoding = Encoding::for_label(b"iso-8859-1").expect("iso-8859-1");
    let (decoded, _, _had_errors) = encoding.decode(bytes);
    decoded.into_owned()
}

fn parse_composite_id(composite_id: &str) -> (i32, String) {
    // [2841, 'Anna M�ller']
    let trimmed = composite_id.replace("[", "").replace("]", "");
    let mut split = trimmed.split(',');
    let id: i32 = split.next().unwrap().trim().parse().expect("i32");
    let name = split.next().unwrap().trim().to_string();
    let name_trimmed = name.trim_start_matches('\'').trim_end_matches('\'');
    (id, name_trimmed.to_string())
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
    Name,
    SaleOrderNumber,
    PartnerId,
    CustomerCompanyId,
    FrameNumber,
    Brand,
    Model,
    Category,
    NetValue,
    StartLeasing,
    EndLeasing,
    Term,
    InsuranceRate,
    #[default]
    Unknown,
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn test_parse_contracts_xml() {
        use crate::ingest::ingest_contracts::parse_contracts_xml;
        //
        let xml = r#"
        <?xml version="1.0" encoding="ISO-8859-1"?>
<export model="lr.contract.leasing" generated_at="2026-07-27 02:14:09" source="erp">
  <record>
    <field name="name">ELV-100234</field>
    <field name="sale_order_number">KAU-003945</field>
    <field name="partner_id">[2841, 'Anna M�ller']</field>
    <field name="customer_company_id">[512, 'Backwaren Gro� GmbH']</field>
    <field name="frame_number">WBL120344</field>
    <field name="brand">Cube</field>
    <field name="model">Kathmandu Hybrid Pro 750</field>
    <field name="category">3499.00</field>
    <field name="net_value">2024-09-01</field>
    <field name="start_leasing">2027-08-31</field>
    <field name="end_leasing">36</field>
    <field name="term">4.53</field>
    <field name="insurance_rate">active</field>
  </record>
  <record>
    <field name="name">ELV-100301</field>
    <field name="sale_order_number">KAU-004102</field>
    <field name="partner_id">[2933, 'Jonas Wei�']</field>
    <field name="customer_company_id">[77, 'Stra�enbau S�d AG']</field>
    <field name="frame_number">CAN99801</field>
    <field name="brand">Canyon</field>
    <field name="model">Pathlite:ON 7</field>
    <field name="category">2199.00</field>
    <field name="net_value">2025-01-01</field>
    <field name="start_leasing">2028-12-31</field>
    <field name="end_leasing">48</field>
    <field name="term">4.20</field>
    <field name="insurance_rate">active</field>
  </record>
  <record>
    <field name="name">ELV-100377</field>
    <field name="sale_order_number">KAU-004311</field>
    <field name="partner_id">[3001, 'Ren� B�hm']</field>
    <field name="customer_company_id">[214, 'Kanzlei B�ttcher &amp; Partner']</field>
    <field name="frame_number">SPZ44120</field>
    <field name="brand">Specialized</field>
    <field name="model">Turbo Vado 4.0</field>
    <field name="category">4650.00</field>
    <field name="net_value">2023-05-01</field>
    <field name="start_leasing">2026-04-30</field>
    <field name="end_leasing">36</field>
    <field name="term">5.80</field>
    <field name="insurance_rate">done</field>
  </record>
  <record>
    <field name="name">ELV-100412</field>
    <field name="sale_order_number">KAU-004489</field>
    <field name="partner_id">[3120, 'Sofia Marquardt']</field>
    <field name="customer_company_id">[88, 'Nordlicht Logistik GmbH']</field>
    <field name="frame_number">GAZ77210</field>
    <field name="brand">Gazelle</field>
    <field name="model">Ultimate C380</field>
    <field name="category">3100.00</field>
    <field name="net_value">2025-03-01</field>
    <field name="start_leasing">False</field>
    <field name="end_leasing">36</field>
    <field name="term">4.34</field>
    <field name="insurance_rate">active</field>
  </record>
  <record>
    <field name="name">ELV-100455</field>
    <field name="sale_order_number">KAU-004521</field>
    <field name="partner_id">[3188, 'Tobias Kr�ger']</field>
    <field name="customer_company_id">[301, 'M�belhaus Sch�fer KG']</field>
    <field name="frame_number">TRK55901</field>
    <field name="brand">Trek</field>
    <field name="model">FX+ 2</field>
    <field name="category">2749.00</field>
    <field name="net_value">2024-06-01</field>
    <field name="start_leasing">2027-05-31</field>
    <field name="end_leasing">36</field>
    <field name="term">4.20</field>
    <field name="insurance_rate">cancel</field>
  </record>
  <record>
    <field name="name">ELV-100489</field>
    <field name="sale_order_number">KAU-004601</field>
    <field name="partner_id">[3250, 'Leonie Hoffmann']</field>
    <field name="customer_company_id">[512, 'Backwaren Gro� GmbH']</field>
    <field name="frame_number">TRK80112</field>
    <field name="brand">Trek</field>
    <field name="model">Powerfly 5</field>
    <field name="category">5200.00</field>
    <field name="net_value">2025-07-01</field>
    <field name="start_leasing">2028-06-30</field>
    <field name="end_leasing">36</field>
    <field name="term">6.10</field>
    <field name="insurance_rate">active</field>
  </record>
  <record>
    <field name="name">ELV-100502</field>
    <field name="sale_order_number">KAU-004666</field>
    <field name="partner_id">[3311, '�mile Laurent']</field>
    <field name="customer_company_id">[402, 'Druckerei Vogel GmbH']</field>
    <field name="frame_number">RSM33417</field>
    <field name="brand">Riese &amp; M�ller</field>
    <field name="model">Charger4 GT</field>
    <field name="category">7350.00</field>
    <field name="net_value">2026-01-01</field>
    <field name="start_leasing">2028-12-31</field>
    <field name="end_leasing">36</field>
    <field name="term">7.94</field>
    <field name="insurance_rate">active</field>
  </record>
  <record>
    <field name="name">ELV-100533</field>
    <field name="sale_order_number">KAU-004712</field>
    <field name="partner_id">[3377, 'Katharina Gro�']</field>
    <field name="customer_company_id">[77, 'Stra�enbau S�d AG']</field>
    <field name="frame_number">CUB10229</field>
    <field name="brand">Cube</field>
    <field name="model">Reaction Hybrid Performance</field>
    <field name="category">1899.00</field>
    <field name="net_value">2025-10-01</field>
    <field name="start_leasing">2028-09-30</field>
    <field name="end_leasing">36</field>
    <field name="term">4.20</field>
    <field name="insurance_rate">active</field>
  </record>
</export>
        "#;

        let contracts = parse_contracts_xml(xml).await.unwrap();

        assert!(!contracts.is_empty());
    }
}
