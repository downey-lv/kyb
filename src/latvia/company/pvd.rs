use crate::latvia::pvd::PvdRecord;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PvdResult {
    pub pvd_code: String,
    pub address: String,
    pub object_name: String,
}

impl From<PvdRecord> for PvdResult {
    fn from(record: PvdRecord) -> Self {
        Self {
            pvd_code: record.pvd_code,
            address: record.address,
            object_name: record.object_name,
        }
    }
}

pub async fn add_pvd_results(
    conn: &Connection,
    companies: &mut Vec<crate::latvia::company::company::Company>,
) -> Result<(), Box<dyn Error>> {
    PvdRecord::create_table(conn).await?;
    let mut cache: HashMap<String, Vec<PvdResult>> = HashMap::new();

    for company in companies {
        let results = match cache.get(&company.reg_code) {
            Some(results) => results.clone(),
            None => {
                let records = PvdRecord::find_by_reg_code(conn, &company.reg_code).await?;
                let mut seen_pvd_codes = HashSet::new();
                let results: Vec<PvdResult> = records
                    .into_iter()
                    .filter_map(|record| {
                        if seen_pvd_codes.insert(record.pvd_code.clone()) {
                            Some(PvdResult::from(record))
                        } else {
                            None
                        }
                    })
                    .collect();
                cache.insert(company.reg_code.clone(), results.clone());
                results
            }
        };

        if !results.is_empty() {
            company.pvd = Some(results);
        }
    }

    Ok(())
}
