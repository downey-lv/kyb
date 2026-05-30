use crate::latvia::company::company::Company;
use rusqlite::{Result, Rows};
use std::error::Error;

struct SearchResult {
    legal_form: String,
    name: String,
    address: Option<String>,
    city: Option<String>,
    zip: Option<String>,
    public_sector: String,
    reg_code: String,
    vat: bool,
    vat_number: Option<String>,
}

pub async fn search_map_results(mut rows: Rows<'_>) -> Result<Vec<Company>, Box<dyn Error>> {
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let result = SearchResult {
            legal_form: row.get(0)?,
            name: row.get(1)?,
            city: row.get(2)?,
            address: row.get(3)?,
            zip: row.get(4)?,
            public_sector: row.get(5)?,
            reg_code: row.get(6)?,
            vat: row.get(7)?,
            vat_number: row.get(8)?,
        };
        results.push(result);
    }

    let mut search_results: Vec<Company> = Vec::new();

    for search_result in results {
        let company = Company {
            legal_form: search_result.legal_form,
            name: search_result.name,
            city: search_result.city,
            address: search_result.address,
            zip: search_result.zip,
            public_sector: search_result.public_sector,
            reg_code: search_result.reg_code,
            vat: search_result.vat,
            vat_number: search_result.vat_number,
            pvd: None,
        };
        search_results.push(company);
    }

    Ok(search_results)
}
