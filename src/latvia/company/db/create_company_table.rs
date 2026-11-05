use crate::latvia::company::company::Company;
use crate::latvia::company::log::create_table::create_company_log_table;
use rusqlite::Connection;
use std::error::Error;

impl Company {
    pub async fn create_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS company (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                legal_form TEXT,
                name TEXT NOT NULL,
                city TEXT,
                address TEXT,
                zip TEXT,
                public_sector TEXT,
                normal_name TEXT NOT NULL,
                reg_code TEXT NOT NULL,
                vat INTEGER,
                vat_number TEXT
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_normal_name ON company (normal_name)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_reg_code ON company (reg_code)",
            [],
        )?;

        create_company_log_table(&conn).await?;

        Ok(())
    }
}
