use actix_web::{error, web, Error, HttpResponse};
use bytes::Bytes;
use std::str;
use serde_json::Value;
use biscuit::*;
use biscuit::jws::*;
use biscuit::jwa::*;
use crate::AppState;

pub async fn create_token(state: web::Data<AppState>, claims_req: Bytes) -> Result<HttpResponse, Error> {
    let signing_secret = &state.rsa_key_pair;

    let res: Result<Value, Error> = str::from_utf8(&claims_req)
        .map(|s| serde_json::from_str(s)).unwrap()
        .map_err(error::ErrorInternalServerError);
      
    //Please note that the way the token is created with RegisteredClaims (all None)
    //and private claims with a JSON Value with all passed claims is a bit of a hack. 
    res.and_then( |claims| { 
        match claims {
            Value::Object(ref _v) => {
                let decoded_token = JWT::new_decoded(From::from(
                    RegisteredHeader {
                        algorithm: SignatureAlgorithm::RS256,
                        ..Default::default()
                    }),
                ClaimsSet::<Value> {
                    registered: RegisteredClaims {
                        issuer: None,
                        subject: None, 
                        audience: None, 
                        not_before: None,
                        expiry: None, 
                        id: None, 
                        issued_at: None,
                    },
                    private: claims,
                });
                let encoded_token = decoded_token.encode(&signing_secret).unwrap().unwrap_encoded().to_string();
                Ok(HttpResponse::Ok().content_type("text/plain").body(encoded_token))
            },
            other => Err(error::ErrorBadRequest(format!("Claims are not given as JSON object but as: {:?}", other))), 
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::Service;
    use actix_web::{http, test, web, App};
    use std::str;

    #[actix_rt::test]
    async fn test_route_create_token() -> Result<(), Error> {

        let claims = r##"
            {
                "iss": "http://localhost:8080/mock",
                "sub": "CgVhZG1pbhIFbG9jYWw",
                "aud": "cafienne-ui",
                "exp": 1576568495,
                "iat": 1576482095,
                "at_hash": "zqKhL-sV6TNJUFQSF7PwLQ",
                "email": "admin@example.com",
                "email_verified": true,
                "name": "admin"
            }
        "##;

        let mut app = test::init_service(
            App::new()
                .data(AppState::new("./static/private_key.der"))
                .service(web::resource("/").route(web::post().to(create_token))),
        ).await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_payload(claims)
            .to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.status(), http::StatusCode::OK);
    
        let response_body = match resp.response().body().as_ref() {
            Some(actix_web::body::Body::Bytes(bytes)) => bytes,
            _ => panic!("Response error"),
        };
    
        let body_str = match str::from_utf8(&response_body) {
            Ok(v) => v,
            Err(_e) => "Error with parsing result from bytes to string",
        };
        
        assert_eq!(body_str, "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJodHRwOi8vbG9jYWxob3N0OjgwODAvbW9jayIsInN1YiI6IkNnVmhaRzFwYmhJRmJHOWpZV3ciLCJhdWQiOiJjYWZpZW5uZS11aSIsImV4cCI6MTU3NjU2ODQ5NSwiaWF0IjoxNTc2NDgyMDk1LCJhdF9oYXNoIjoienFLaEwtc1Y2VE5KVUZRU0Y3UHdMUSIsImVtYWlsIjoiYWRtaW5AZXhhbXBsZS5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwibmFtZSI6ImFkbWluIn0.hMp7HYL_nlsnf_Q1XFDWo_dbwFfdSg70yK9BHzN_nykregJ7oa2GVPT9VlrjrfncH4YJwBs9fMSRxAEXa3lfXLHLcZFrd6r4Kxhfrsl9vFmuIxlbJJhJj-_0uDnyVRatMlMHfnJsJZCqKIeE6BB2xopdgyGsNuhHx1bxXGb-Ty5Da0OCHIdCgpTfxzrtPDs87saT0i-4ohBbCIKeA0Smxu3wHFQaLH1qWO5vvsW_bAT35agHgO7rIGkK0Bf-hMba3RUe3R4VQA7Pgr56CzwZnhMpiJNeISdQTUZss8P5QovG_4SUIWTb6ybioyMFwXMiTWb3oWXWwX5yZnvTRM-khA");

        Ok(())
    }
}
