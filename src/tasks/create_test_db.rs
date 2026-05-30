use crate::db::get_db::Pool;
use crate::latvia::company::company::Company;
use crate::latvia::company::import::import::import_companies_from_csv;
use crate::latvia::government::import::import_public_institutions_from_csv;
use crate::latvia::pvd::import_pvd_from_csv;
use crate::latvia::vat::read_sample_data::read_sample_vat_data;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};

#[allow(dead_code)]
pub async fn create_test_db() -> Result<Pool, Box<dyn Error>> {
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::new(manager).unwrap();
    let mut conn: PooledConnection<SqliteConnectionManager> =
        pool.get().expect("Couldn't get db connection from pool");

    // ADD TABLE SCHEMA
    Company::create_table(&conn).await?;
    // GET SAMPLE DATA
    let path = "./src/latvia/company/import/company.csv";
    let mut file = File::open(path)?;
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents)?;
    let cursor = Cursor::new(contents);

    let rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(cursor);
    // GET VAT DATA
    let vat_table = read_sample_vat_data().await?;
    // IMPORT COMPANIES
    import_companies_from_csv(&mut conn, rdr, &vat_table).await?;
    // GET PUBLIC COMPANIES DATA
    let path_gov = "./src/latvia/government/ppi_public_persons_institutions.csv";
    let mut file_gov = File::open(path_gov)?;
    let mut contents_gov = String::new();
    file_gov.read_to_string(&mut contents_gov)?;
    let cursor_gov = Cursor::new(contents_gov);

    let rdr_gov = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(cursor_gov);

    import_public_institutions_from_csv(&mut conn, rdr_gov, &vat_table).await?;
    // GET PVD DATA
    let path_pvd = "./src/latvia/pvd/ur-dati.csv";
    let mut file_pvd = File::open(path_pvd)?;
    let mut contents_pvd = String::new();
    file_pvd.read_to_string(&mut contents_pvd)?;
    let cursor_pvd = Cursor::new(contents_pvd);

    let rdr_pvd = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .trim(csv::Trim::All)
        .from_reader(cursor_pvd);

    import_pvd_from_csv(&mut conn, rdr_pvd).await?;
    // IMPORT CSB
    Ok(pool)
}
