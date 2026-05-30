use crate::config::KybConfig;
use csv::Reader;
use reqwest::get;
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Cursor, Read};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PvdRecord {
    pub reg_code: String,
    pub pvd_code: String,
    pub address: String,
    pub object_name: String,
}

#[derive(Debug, Deserialize)]
struct PvdCsvRecord {
    #[serde(rename = "UznemumaRegistracijasNr")]
    reg_code: String,
    #[serde(rename = "ObjektaNosaukums")]
    object_name: String,
    #[serde(rename = "ObjektaPvdNr")]
    pvd_code: String,
    #[serde(rename = "ObjektaAdrese")]
    address: String,
}

impl PvdRecord {
    pub async fn create_table(conn: &Connection) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pvd (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                reg_code TEXT NOT NULL,
                pvd_code TEXT NOT NULL,
                address TEXT NOT NULL,
                object_name TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pvd_reg_code ON pvd (reg_code)",
            [],
        )?;

        Ok(())
    }

    pub async fn find_by_reg_code(
        conn: &Connection,
        reg_code: &str,
    ) -> Result<Vec<PvdRecord>, Box<dyn Error>> {
        let mut stmt = conn.prepare(
            "SELECT reg_code, pvd_code, address, object_name
            FROM pvd
            WHERE reg_code = ?1
            ORDER BY object_name, pvd_code",
        )?;
        let rows = stmt.query_map(params![reg_code], |row| {
            Ok(PvdRecord {
                reg_code: row.get(0)?,
                pvd_code: row.get(1)?,
                address: row.get(2)?,
                object_name: row.get(3)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}

pub async fn import_pvd_from_csv(
    conn: &mut Connection,
    mut rdr: Reader<Cursor<String>>,
) -> Result<(), Box<dyn Error>> {
    PvdRecord::create_table(conn).await?;
    conn.execute("DELETE FROM pvd", [])?;

    let transaction = conn.transaction()?;

    {
        let mut stmt = transaction.prepare(
            "INSERT INTO pvd (reg_code, pvd_code, address, object_name)
            VALUES (?1, ?2, ?3, ?4)",
        )?;

        for result in rdr.deserialize() {
            let record: PvdCsvRecord = result?;
            let reg_code = record.reg_code.trim();
            let object_name = record.object_name.trim();
            let pvd_code = record.pvd_code.trim();
            let address = record.address.trim();

            if !reg_code.is_empty() && !pvd_code.is_empty() {
                stmt.execute(params![reg_code, pvd_code, address, object_name])?;
            }
        }
    }

    transaction.commit()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{PvdRecord, import_pvd_from_csv};
    use rusqlite::Connection;
    use std::io::Cursor;

    #[actix_rt::test]
    async fn pvd_import_reads_current_header() {
        let mut conn = Connection::open_in_memory().unwrap();
        let contents = concat!(
            "Nosaukums\tUznemumaRegistracijasNr\tObjektaNosaukums\tObjektaPvdNr\tObjektaAdrese\tDarbibasVeidaNosaukums\tDarbibasVeidaKods\tDarbibasVeidaAtzisanasVertiba\tDarbibasVeidaAtzinumsArNosacijumuSpekaLidzDatums\tDarbibasVeidaPedejasPlanveidaParbaudesVertejums\n",
            "Test company\t40203572370\tProduction kitchen\tPVD-123\tFantastics prospekts 123\tFood handling\t10\t-\t\t\n",
            "Test company\t40203572370\tProduction kitchen\tPVD-123\tFantastics prospekts 123\tFood storage\t20\t-\t\t\n",
        );
        let cursor = Cursor::new(contents.to_string());
        let rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .trim(csv::Trim::All)
            .from_reader(cursor);

        import_pvd_from_csv(&mut conn, rdr).await.unwrap();

        let results = PvdRecord::find_by_reg_code(&conn, "40203572370")
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].pvd_code, "PVD-123");
        assert_eq!(results[0].address, "Fantastics prospekts 123");
        assert_eq!(results[0].object_name, "Production kitchen");
    }
}

pub async fn fetch_new_pvd_data(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let url = KybConfig::SOURCE_PVD;
    println!("getting {}", url);
    let response = get(url).await?.bytes().await?;
    let reader = Cursor::new(response);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut file = archive.by_name("ur-dati.csv")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    drop(file);

    let cursor = Cursor::new(contents);
    let rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .trim(csv::Trim::All)
        .from_reader(cursor);

    import_pvd_from_csv(conn, rdr).await?;

    Ok(())
}
