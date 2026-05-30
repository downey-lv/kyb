use crate::db::get_db::Pool;
use crate::error::KybError;
use crate::latvia::company::search::lv_company_search_handle::CompanySearchQuery;
use crate::latvia::company::search::lv_company_search_handle::lv_company_search_handle;
use actix_web::{HttpResponse, Responder, web};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

pub async fn lv_company_search(
    pool: web::Data<Pool>,
    query: web::Json<CompanySearchQuery>,
) -> impl Responder {
    let conn: PooledConnection<SqliteConnectionManager> =
        pool.get().expect("Couldn't get db connection from pool");

    let company_query: CompanySearchQuery = query.into_inner();
    match lv_company_search_handle(&conn, company_query).await {
        Ok(results) => {
            // drop(pool);
            return HttpResponse::Ok().json(results);
        }
        Err(err) => {
            //  drop(pool);
            match err {
                KybError::StringError(err) => {
                    eprintln!("kyb err: {:?}", &err);
                    HttpResponse::ExpectationFailed().json(serde_json::json!({ "error": err }))
                }
                _ => {
                    eprintln!("server down: {:?}", &err);
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Server down" }))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        latvia::{company::company::Company, pvd::PvdRecord},
        tasks::create_test_db::create_test_db,
    };
    use actix_web::{App, test, web};
    use reqwest::Client;
    use rusqlite::params;
    use serde_json::json;
    use std::time::{Duration, Instant};

    // THIS ASSUMES DB IS INSTALLED
    #[actix_rt::test]
    async fn test_lv_company_search_success() {
        // Create DB
        let db = create_test_db().await.unwrap();
        // Arrange: Set up the test server and request payload

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .route("/lv/company", web::post().to(lv_company_search)),
        )
        .await;

        let request_payload = json!({
            "name": "Raimond fantastic"
        });

        let req = test::TestRequest::post()
            .uri("/lv/company")
            .set_json(&request_payload)
            .to_request();

        let start = Instant::now();

        // Act: Send the request to the test server
        let resp = test::call_service(&mut app, req).await;
        let duration = start.elapsed();
        assert!(
            duration < Duration::from_millis(50),
            "Operation took too long: {:?}",
            duration
        );

        println!("duration {:?}", duration);
        println!("{:?}", resp);

        // Assert: Check the response
        assert!(resp.status().is_success());
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body[0]["reg_code"], "40203572370");
        assert!(response_body[0].get("pvd").is_none());
    }

    #[actix_rt::test]
    async fn test_lv_company_search_with_pvd() {
        let db = create_test_db().await.unwrap();

        {
            let mut conn = db.get().expect("Couldn't get db connection from pool");
            PvdRecord::create_table(&conn).await.unwrap();
            let transaction = conn.transaction().unwrap();
            transaction
                .execute(
                    "INSERT INTO pvd (reg_code, pvd_code, address, object_name)
                    VALUES (?1, ?2, ?3, ?4)",
                    params![
                        "40203572370",
                        "PVD-123",
                        "Fantastics prospekts 123",
                        "Production kitchen"
                    ],
                )
                .unwrap();
            transaction
                .execute(
                    "INSERT INTO pvd (reg_code, pvd_code, address, object_name)
                    VALUES (?1, ?2, ?3, ?4)",
                    params![
                        "40203572370",
                        "PVD-123",
                        "Fantastics prospekts 123",
                        "Production kitchen"
                    ],
                )
                .unwrap();
            transaction.commit().unwrap();
        }

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .route("/lv/company", web::post().to(lv_company_search)),
        )
        .await;

        let request_payload = json!({
            "name": "Raimond fantastic",
            "include_pvd": true
        });

        let req = test::TestRequest::post()
            .uri("/lv/company")
            .set_json(&request_payload)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body[0]["reg_code"], "40203572370");
        assert_eq!(response_body[0]["pvd"].as_array().unwrap().len(), 1);
        assert_eq!(response_body[0]["pvd"][0]["pvd_code"], "PVD-123");
        assert_eq!(
            response_body[0]["pvd"][0]["object_name"],
            "Production kitchen"
        );
        assert_eq!(
            response_body[0]["pvd"][0]["address"],
            "Fantastics prospekts 123"
        );
    }

    #[actix_rt::test]
    async fn test_lv_company_search_failure() {
        let db = create_test_db().await.unwrap();

        // Arrange: Set up the test server and request payload
        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .route("/lv/company", web::post().to(lv_company_search)),
        )
        .await;

        let request_payload = json!({
            "name": ""
        });

        let req = test::TestRequest::post()
            .uri("/lv/company")
            .set_json(&request_payload)
            .to_request();

        // Act: Send the request to the test server
        let resp = test::call_service(&mut app, req).await;
        // ASSERT IT'S 200
        assert!(resp.status().is_success());
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        // ASSERT IT'S EMPTY
        assert_eq!(response_body, json!([]));
    }
    // THIS ASSUMES SERVER IS RUNNING
    #[test]
    async fn integration_lv_company_search_1000_times() {
        let client = Client::new();
        let url = "http://localhost:10001/lv/company";
        let payload = json!({ "name": "Raimond fantastic" });
        for _ in 0..1000 {
            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .body(payload.to_string())
                .send()
                .await
                .expect("Failed to send request");

            assert!(response.status().is_success(), "Request failed");

            let response_body = response.text().await.expect("Failed to read response body");
            let parsed: Vec<Company> = serde_json::from_str(&response_body).unwrap();

            assert_eq!(parsed[0].reg_code, "40203572370".to_string());
        }
    }
}
