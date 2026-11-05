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
                KybError::BadRequest(err) => {
                    HttpResponse::BadRequest().json(serde_json::json!({ "error": err }))
                }
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
    async fn post_company(
        db: crate::db::get_db::Pool,
        payload: serde_json::Value,
    ) -> (actix_web::http::StatusCode, serde_json::Value) {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .route("/lv/company", web::post().to(lv_company_search)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/lv/company")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let body: serde_json::Value = test::read_body_json(resp).await;
        (status, body)
    }

    #[actix_rt::test]
    async fn test_lv_company_reg_code_exact_match() {
        let db = create_test_db().await.unwrap();
        let (status, body) = post_company(db, json!({ "reg_code": "40203572370" })).await;

        assert_eq!(status, 200);
        let results = body.as_array().unwrap();
        assert_eq!(results.len(), 1);
        let company = &results[0];
        assert_eq!(company["reg_code"], "40203572370");
        assert_eq!(company["name"], "Raimond fantastic");
        assert_eq!(company["legal_form"], "SIA");
        assert_eq!(company["city"], "Jūrmala");
        assert_eq!(company["address"], "Mellužu prospekts 76");
        assert_eq!(company["public_sector"], "0");
        assert_eq!(company["vat"], true);
        assert_eq!(company["vat_number"], "LV40203572370");
        assert!(company.get("pvd").is_none());
    }

    #[actix_rt::test]
    async fn test_lv_company_reg_code_lv_prefix_is_stripped() {
        let db = create_test_db().await.unwrap();
        for code in ["LV40203572370", "lv40203572370", " LV40203572370 "] {
            let (status, body) = post_company(db.clone(), json!({ "reg_code": code })).await;
            assert_eq!(status, 200, "{code}");
            assert_eq!(body[0]["reg_code"], "40203572370", "{code}");
        }
    }

    #[actix_rt::test]
    async fn test_lv_company_reg_code_is_exact_not_partial() {
        let db = create_test_db().await.unwrap();
        // UNKNOWN CODE
        let (status, body) = post_company(db.clone(), json!({ "reg_code": "40000000000" })).await;
        assert_eq!(status, 200);
        assert_eq!(body, json!([]));
        // LIQUIDATED COMPANY IS NOT IN THE DB
        let (status, body) = post_company(db, json!({ "reg_code": "44102037886" })).await;
        assert_eq!(status, 200);
        assert_eq!(body, json!([]));
    }

    #[actix_rt::test]
    async fn test_lv_company_query_validation_is_400() {
        let db = create_test_db().await.unwrap();
        let bad = [
            json!({}),
            json!({ "include_pvd": true }),
            json!({ "name": "Raimond fantastic", "reg_code": "40203572370" }),
            json!({ "reg_code": "4020357237" }),    // 10
            json!({ "reg_code": "402035723700" }),  // 12
            json!({ "reg_code": "LV4020357237" }),  // 10 AFTER PREFIX
            json!({ "reg_code": "" }),
        ];
        for payload in bad {
            let (status, body) = post_company(db.clone(), payload.clone()).await;
            assert_eq!(status, 400, "{payload}");
            assert!(body["error"].is_string(), "{payload}");
        }
    }

    #[actix_rt::test]
    async fn test_lv_company_reg_code_with_pvd() {
        let db = create_test_db().await.unwrap();
        {
            let conn = db.get().expect("Couldn't get db connection from pool");
            PvdRecord::create_table(&conn).await.unwrap();
            conn.execute(
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
        }
        let (status, body) = post_company(
            db,
            json!({ "reg_code": "LV40203572370", "include_pvd": true }),
        )
        .await;
        assert_eq!(status, 200);
        assert_eq!(body[0]["reg_code"], "40203572370");
        assert_eq!(body[0]["pvd"].as_array().unwrap().len(), 1);
        assert_eq!(body[0]["pvd"][0]["pvd_code"], "PVD-123");
    }

    #[actix_rt::test]
    async fn test_lv_company_reg_code_db_error_is_500_not_empty() {
        // EVERY POOLED CONNECTION IS A FRESH IN-MEMORY DB WITHOUT A company TABLE
        let broken = crate::db::get_db::Pool::new(SqliteConnectionManager::memory()).unwrap();

        let (status, body) = post_company(broken.clone(), json!({ "reg_code": "40203572370" })).await;
        assert_eq!(status, 500);
        assert!(body["error"].is_string());

        // CONTRAST: THE NAME SEARCH STILL SWALLOWS THE SAME ERROR INTO []
        let (status, body) = post_company(broken, json!({ "name": "Raimond fantastic" })).await;
        assert_eq!(status, 200);
        assert_eq!(body, json!([]));
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
