use serde::{Deserialize, Serialize};

use crate::latvia::company::pvd::PvdResult;

#[derive(Debug, Deserialize, Serialize, Clone)]
/// Main constructor for company register
pub struct Company {
    pub legal_form: String,
    pub name: String,
    pub city: Option<String>,
    pub address: Option<String>,
    pub zip: Option<String>,
    pub public_sector: String,
    pub reg_code: String,
    pub vat: bool,
    pub vat_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pvd: Option<Vec<PvdResult>>,
}
