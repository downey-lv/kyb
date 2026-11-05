use crate::error::KybError;
use crate::latvia::company::company::Company;
use crate::latvia::company::log::log::log_search;
use crate::latvia::company::pvd::add_pvd_results;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Exactly one of `name` / `reg_code` must be given.
#[derive(Deserialize, Serialize, Debug)]
pub struct CompanySearchQuery {
    pub name: Option<String>,
    pub reg_code: Option<String>,
    pub include_pvd: Option<bool>,
}

enum SearchBy {
    Name(String),
    RegCode(String),
}

fn search_by(query: &CompanySearchQuery) -> Result<SearchBy, KybError> {
    match (&query.name, &query.reg_code) {
        (Some(name), None) => Ok(SearchBy::Name(name.clone())),
        (None, Some(reg_code)) => Ok(SearchBy::RegCode(normalize_reg_code(reg_code)?)),
        _ => Err(KybError::BadRequest(
            "Exactly one of \"name\" or \"reg_code\" is required".to_string(),
        )),
    }
}

/// Strips an optional "LV" prefix (any case) and requires exactly 11 characters.
fn normalize_reg_code(input: &str) -> Result<String, KybError> {
    let trimmed = input.trim();
    let code = match trimmed.get(..2) {
        Some(prefix) if prefix.eq_ignore_ascii_case("LV") => &trimmed[2..],
        _ => trimmed,
    };
    if code.chars().count() != 11 {
        return Err(KybError::BadRequest(
            "\"reg_code\" must be 11 characters (optional \"LV\" prefix)".to_string(),
        ));
    }
    Ok(code.to_string())
}

pub async fn lv_company_search_handle(
    db: &Connection,
    query: CompanySearchQuery,
) -> Result<Vec<Company>, KybError> {
    let mut results = match search_by(&query)? {
        SearchBy::Name(name) => match Company::search_by_name(db, &name, false).await {
            Ok(results) => results,
            Err(err) => {
                eprintln!("lv_company_search_handle {}", err);
                log_search(&db, &name, &"".to_string(), &vec![], err.to_string()).await;
                return Ok(vec![]);
            }
        },
        // DB ERRORS ARE NOT SWALLOWED HERE: THEY BECOME 500
        SearchBy::RegCode(reg_code) => Company::search_by_reg_code(db, &reg_code)
            .await
            .map_err(|err| {
                eprintln!("lv_company_search_handle reg_code {}", err);
                KybError::StdError(err)
            })?,
    };

    if query.include_pvd.unwrap_or(false) {
        add_pvd_results(db, &mut results).await.map_err(|err| {
            eprintln!("lv_company_search_handle pvd {}", err);
            KybError::StringError(err.to_string())
        })?;
    }

    Ok(results)
}
