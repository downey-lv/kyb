use crate::latvia::board::get_new_data::fetch_and_store_data;
use crate::latvia::company::import::get_new_company_data::fetch_new_company_data;
use crate::latvia::government::get_new_data::fetch_new_public_institution_data;
use crate::latvia::pvd::fetch_new_pvd_data;
use crate::latvia::vat::get_new_data::fetch_new_vat_data;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use std::error::Error;

pub async fn import_new_data(pool: Pool<SqliteConnectionManager>) -> Result<(), Box<dyn Error>> {
    // get db pool
    let mut conn: PooledConnection<SqliteConnectionManager> =
        pool.get().expect("Couldn't get db connection from pool");
    // GET MEMBERS OF BOARDS
    fetch_and_store_data(&mut conn).await?;
    // GET VAT DATA
    let vat_table = fetch_new_vat_data().await?;
    // GET COMPANIES
    fetch_new_company_data(&mut conn, &vat_table).await?;
    // GET PUBLIC INSTITUTIONS
    fetch_new_public_institution_data(&mut conn, &vat_table).await?;
    // GET PVD REGISTRATION DATA
    fetch_new_pvd_data(&mut conn).await?;
    // drop(pool);
    Ok(())
}
