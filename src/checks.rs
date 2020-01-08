use actix_web::{Error, HttpResponse};
use serde_derive::{Deserialize, Serialize};
use sysinfo::SystemExt;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    /*
     {
      "status": "OK",
      "totalMemory" : "",
      "usedMemory" : ""
    }
     */
    pub status: &'static str,
    pub total_memory: Option<String>,
    pub used_memory: Option<String>
}

pub async fn check() -> Result<HttpResponse, Error> {
        let mut system = sysinfo::System::new();
        system.refresh_all();

        let response = HealthResponse {
          status : "OK",
          total_memory : Some(system.get_total_memory().to_string()),
          used_memory : Some(system.get_used_memory().to_string())
        };

        Ok(HttpResponse::Ok().json(response))
}


#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};
    use serde_json::json;
    use serde_json::Value;
    use std::str;

    #[actix_rt::test]
    async fn test_route_check() -> Result<(), Error> {
        let mut app = test::init_service(
        App::new()
                .service(web::resource("/health").route(web::get().to(check))),
        )
        .await;

        let count1_request = test::TestRequest::get()
            .uri("/health")
            .header("Content-Type", "application/json")
            .to_request();

        let resp = app.call(count1_request).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);
    
        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };
    
        let body_str = match str::from_utf8(&response_body) {
            Ok(v) => v,
            Err(_e) => "Error with parsing result from bytes to string",
        };
        let p: Value = serde_json::from_str(body_str).unwrap();

        println!("Value : {:?}", p);

        assert_eq!(p["status"], json!("OK"));

        Ok(())
    }

}
