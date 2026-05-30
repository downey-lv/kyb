use crate::error::KybError;
use crate::latvia::company::company::Company;
use crate::latvia::company::log::log::log_search;
use crate::latvia::company::pvd::add_pvd_results;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CompanySearchQuery {
    pub name: String,
    pub include_pvd: Option<bool>,
}

pub async fn lv_company_search_handle(
    db: &Connection,
    query: CompanySearchQuery,
) -> Result<Vec<Company>, KybError> {
    match Company::search_by_name(db, &query.name, false).await {
        Ok(mut results) => {
            if query.include_pvd.unwrap_or(false) {
                add_pvd_results(db, &mut results).await.map_err(|err| {
                    eprintln!("lv_company_search_handle pvd {}", err);
                    KybError::StringError(err.to_string())
                })?;
            }

            return Ok(results);
        }
        Err(err) => {
            eprintln!("lv_company_search_handle {}", err);
            log_search(&db, &query.name, &"".to_string(), &vec![], err.to_string()).await;
            let empty = vec![];
            return Ok(empty);
        }
    };
}
