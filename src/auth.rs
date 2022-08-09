use std::ops::Add;
use crate::AppState;
use actix_web::{web, Error, HttpResponse};
use actix_web::http::StatusCode;
use serde_derive::Deserialize;
use serde_json::json;
use std::time::SystemTime;
use der_parser::nom::Slice;
use ring::digest;
use data_encoding::BASE64URL_NOPAD;

#[derive(Deserialize)]
pub struct AuthParameters {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
    state: String,
    nonce: String,
}

pub async fn auth(
    state: web::Data<AppState>,
    info: web::Query<AuthParameters>,
) -> Result<HttpResponse, Error> {
    let body = format!(include_str!("../template/login.html"), state = info.state, redirect_uri = info.redirect_uri, nonce = info.nonce, client_id = info.client_id);
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body))
}


#[derive(Deserialize)]
pub struct LoginParameters {
    state: String,
    client_id: String,
    redirect_uri: String,
    sub: String,
    nonce: String,
    name: String,
}
pub async fn login(
    app_state: web::Data<AppState>,
    form: web::Form<LoginParameters>,
) -> Result<HttpResponse, Error> {
    let signing_secret = &app_state.rsa_key_pair;

    // Setup a a series of claims and corrections.
    let iat = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let exp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().add(std::time::Duration::from_secs(12200)).as_secs();

    // Create the access token
    let access_claims = json!(
            {
                "iss": app_state.exposed_host,
                "sub": form.sub,
                "aud": form.client_id,
                "name": form.name,
                "iat": iat,
                "exp": exp
            }
        );
    let access_token = crate::token::create_jwt(&signing_secret, access_claims);

    // at_hash. Access Token hash value.
    // Its value is the base64url encoding of the left-most half of the hash of the octets of the ASCII representation of the access_token value,
    // where the hash algorithm used is the hash algorithm used in the alg Header Parameter of the ID Token's JOSE Header.
    // For instance, if the alg is RS256, hash the access_token value with SHA-256, then take the left-most 128 bits and base64url encode them. (without padding)
    // The at_hash value is a case sensitive string.
    let sha_digest = digest::digest(&digest::SHA256, access_token.as_bytes());
    let tb_encoded = sha_digest.as_ref();
    let at_hash = BASE64URL_NOPAD.encode(tb_encoded.slice(0..16));

    let id_claims = json!(
            {
                "iss": app_state.exposed_host,
                "sub": form.sub,
                "aud": form.client_id,
                "name": form.name,
                "iat": iat,
                "exp": exp,
                "nonce": form.nonce,
                "at_hash": at_hash
            }
        );
    let id_token = crate::token::create_jwt(&signing_secret, id_claims);

    Ok(HttpResponse::build(StatusCode::SEE_OTHER)
        .insert_header(("Location", format!("{redirect_uri}#access_token={access_token}&expires_in=86399&id_token={id_token}&state={state}&token_type=bearer", redirect_uri = form.redirect_uri, access_token = access_token, id_token = id_token, state = form.state)))
        .finish()
    )
}

