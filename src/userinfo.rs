use crate::discovery::create_jwk_set;
use actix_4_jwt_auth::{
    AuthenticatedUser, Oidc, OidcConfig, OidcBiscuitValidator, 
    biscuit::{ValidationOptions, Validation}
};
use actix_web::{web, Error, HttpResponse};
use biscuit::jws::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FoundClaims {
    pub sub: String,
    pub name: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
}

pub async fn user_info(user: AuthenticatedUser<FoundClaims>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(user.claims))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token;
    use actix_web::{http, test, App};
    use biscuit::ValidationOptions;
    use serde_json::json;
    use std::str;

    #[actix_rt::test]
    async fn test_route_userinfo() -> Result<(), Error> {
        let claims = r##"
            {
                "iss": "http://localhost:8080",
                "sub": "F82E617D-DEAF-4EE6-8F96-CF3409060CA2",
                "aud": "oidc-token-mock",
                "email": "admin@example.com",
                "email_verified": true,
                "name": "Arie Ministrone"
            }
        "##;

        let validation_options = ValidationOptions::default();
        let rsa_keys = Secret::rsa_keypair_from_file("./keys/private_key.der")
            .expect("Cannot read RSA keypair");
        let jwk_set = create_jwk_set(rsa_keys.clone());
        let issuer = "http://localhost:8080".to_string();
        // let oidc_validator = OIDCValidator::new_for_jwks(jwk_set, validation_options)
        //     .await
        //     .unwrap();

        let oidc = Oidc::new(OidcConfig::Issuer(issuer.clone().into())).await.unwrap();

        let biscuit_validator = OidcBiscuitValidator { options: ValidationOptions {
                issuer: Validation::Validate(issuer),
                ..ValidationOptions::default()
            }
        };
        let claims_json = serde_json::from_str(claims).unwrap();
        let jwt = token::create_jwt(&rsa_keys, claims_json);

        let app = test::init_service(
            App::new()
                .app_data(oidc.clone())
                .wrap(biscuit_validator.clone())
                // .wrap(OidcBiscuitValidator::default())
                // .app_data(OIDCValidatorConfig {
                //     issuer: issuer.clone(),
                //     validator: oidc_validator.clone(),
                // })
                .service(web::resource("/").route(web::post().to(user_info))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/")
            .insert_header(("Authorization", format!("Bearer {}", jwt)))
            .set_payload(claims)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);

        let response_body = test::read_body(resp).await;
        let body_str = match str::from_utf8(&response_body) {
            Ok(v) => v,
            Err(_e) => "Error with parsing result from bytes to string",
        };
        let p: Value = serde_json::from_str(body_str).unwrap();

        assert_eq!(p["name"], json!("Arie Ministrone"));

        Ok(())
    }
}
