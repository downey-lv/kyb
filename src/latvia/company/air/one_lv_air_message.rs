use crate::db::get_db::Pool;
use crate::error::KybError;
use crate::latvia::company::air::air_traffic::AirSearchResponse;
use crate::latvia::company::air::air_traffic::Ping;
use crate::latvia::company::air::air_traffic::Pong;
use crate::latvia::company::air::air_traffic::send_message;
use crate::latvia::company::search::lv_company_search_handle::CompanySearchQuery;
use crate::latvia::company::search::lv_company_search_handle::lv_company_search_handle;
use actix_web::web;
use actix_ws::AggregatedMessage;
use actix_ws::ProtocolError;
use actix_ws::Session;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn one_lv_air_message(
    pool: web::Data<Pool>,
    mut session: &mut Session,
    msg: Result<AggregatedMessage, ProtocolError>,
) -> Result<(), KybError> {
    // HANDLE MSG AND PING
    match msg {
        Ok(AggregatedMessage::Text(bytes)) => {
            // HANDLE ONE MESSAGE
            let as_v8 = bytes.as_bytes();
            // PING IS CHECKED FIRST: EVERY SEARCH FIELD IS OPTIONAL, SO A PING ALSO PARSES AS A SEARCH
            let ping = serde_json::from_slice::<Ping>(&as_v8);
            let search = serde_json::from_slice::<CompanySearchQuery>(&as_v8);
            if ping.is_ok() {
                // THIS IS PING
                let pong = Pong { pong: true };
                let payload = serde_json::to_string(&pong)?;
                if let Err(e) = &session.text(payload).await {
                    return Err(KybError::StringError(e.to_string()));
                }
            } else if search.is_ok() {
                // SOMEONE IS ACTUALLY SEARCHING SOMETHING - GET DB
                let conn: PooledConnection<SqliteConnectionManager> =
                    pool.get().expect("Couldn't get db connection from pool");
                // EXTRACT SEARCH QUERY
                let query = search.unwrap();
                // SEARCH; INVALID QUERIES ARE ANSWERED, NOT DROPPED
                let respo = match lv_company_search_handle(&conn, query).await {
                    Ok(result) => AirSearchResponse {
                        result: Some(result),
                        error: None,
                    },
                    Err(KybError::BadRequest(err)) => AirSearchResponse {
                        result: None,
                        error: Some(err),
                    },
                    Err(err) => return Err(err),
                };
                // SEND
                send_message(&mut session, respo).await?;
            } else {
                // IT'S NEIGHTER OF THE TWO
                let respo = AirSearchResponse {
                    error: Some("Failed to deserialize message".to_string()),
                    result: None,
                };
                send_message(&mut session, respo).await?;
            }

            Ok(())
        }
        // ALL OTHER CHATTER IS IGNORED
        _ => Ok(()),
    }
}
