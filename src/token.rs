use crate::AppState;
use actix_web::{error, web, Error, HttpResponse};
use biscuit::jwa::*;
use biscuit::jws::*;
use biscuit::*;
use bytes::Bytes;
use serde_json::Value;
use std::str;

pub async fn create_token(
    state: web::Data<AppState>,
    claims_req: Bytes,
) -> Result<HttpResponse, Error> {
    let signing_secret = &state.rsa_key_pair;

    let res: Result<Value, Error> = str::from_utf8(&claims_req)
        .map(|s| serde_json::from_str(s))
        .unwrap()
        .map_err(error::ErrorInternalServerError);

    //Please note that the way the token is created with RegisteredClaims (all None)
    //and private claims with a JSON Value with all passed claims is a bit of a hack.
    res.and_then(|claims| match claims {
        Value::Object(ref _v) => {
            let encoded_token = create_jwt(&signing_secret, claims);
            Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body(encoded_token))
        }
        other => Err(error::ErrorBadRequest(format!(
            "Claims are not given as JSON object but as: {:?}",
            other
        ))),
    })
}

pub fn create_jwt(signing_secret: &Secret, claims: Value) -> String {
    let decoded_token = JWT::new_decoded(
        From::from(RegisteredHeader {
            algorithm: SignatureAlgorithm::RS256,
            key_id: Some("2020-01-29".to_string()),
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
        },
    );
    decoded_token
        .encode(&signing_secret)
        .unwrap()
        .unwrap_encoded()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let exposed_host = "http://localhost:8080".to_string();
        let rsa_keys = Secret::rsa_keypair_from_file("./keys/private_key.der")
            .expect("Cannot read RSA keypair");
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState::new(
                    &rsa_keys,
                    exposed_host,
                )))
                .service(web::resource("/").route(web::post().to(create_token))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/")
            .set_payload(claims)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = test::read_body(resp).await;
        let body_str = match str::from_utf8(&response_body) {
            Ok(v) => v,
            Err(_e) => "Error with parsing result from bytes to string",
        };

        assert_eq!(body_str, "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IjIwMjAtMDEtMjkifQ.eyJpc3MiOiJodHRwOi8vbG9jYWxob3N0OjgwODAvbW9jayIsInN1YiI6IkNnVmhaRzFwYmhJRmJHOWpZV3ciLCJhdWQiOiJjYWZpZW5uZS11aSIsImV4cCI6MTU3NjU2ODQ5NSwiaWF0IjoxNTc2NDgyMDk1LCJhdF9oYXNoIjoienFLaEwtc1Y2VE5KVUZRU0Y3UHdMUSIsImVtYWlsIjoiYWRtaW5AZXhhbXBsZS5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwibmFtZSI6ImFkbWluIn0.KxJNef8u8N8t7CfHSiha4yFpiivRGcR_zmNNAN9CJGBGuX5i0h9cYw1AGupNvBe5VEQTpp_hk3_S5lJE8qTw60ey9zUfbbiMX3uWUUsqNVcCv51kF5hzPA0eQffZMpMRBSzJa1WgY39yQATy2eBoDEt_JPXixGOy6Xl9Op9VoDozFyVYtG31oUSM4rFhSqTAYFrRfXIdrYIaBkcqd5FFRRidSb6mSgZwl9YT5gCr2LF7fLAePqAEJqiQP3weOJNytv52OMRMjosmO6bnQQvNx6Hq7M3o6n-nfWa8SE7GlvV4MJ8b-HR8n6xQ4EZYZ09hBM2HYlS1CqpAjHs0OM3z9g");

        Ok(())
    }
}
