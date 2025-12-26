use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub amount: f64,
    pub product: String,
    pub jurisdiction: String,
    pub client_id: String,
    pub date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub client_id: String,
    pub fiscal_category: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaxBreakdown {
    pub tax_type: TaxType,
    pub base: f64,
    pub rate: f64,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IvaRate {
    pub jurisdiction: String,
    pub rate: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TaxType {
    IVA,
    Sellos,
    IIBB,
    Ganancias,
}
