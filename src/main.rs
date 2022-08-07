use actix_web::{middleware, web, App, HttpServer};
use biscuit::jws::Secret;
use clap;
use std::process::Command;
use actix_4_jwt_auth::{OIDCValidator, OIDCValidatorConfig};

mod checks;
mod discovery;
mod token;
mod userinfo;

//AppState object is initialized for the App and passed with every request that has a parameter with the AppState as type.
pub struct AppState {
    rsa_key_pair: biscuit::jws::Secret,
    exposed_host: String,
}

impl AppState  {
    pub fn new(rsa_keys: &Secret, exposed_host: String) -> Self {
        Self {
            rsa_key_pair: rsa_keys.clone(),
            exposed_host: exposed_host.clone(),
        }
    }
}

/*
Profiling: http://carol-nichols.com/2015/12/09/rust-profiling-on-osx-cpu-time/
*/
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = clap::App::new("oidc-token-test-service")
        .version("0.2")
        .about("Allows to generate any valid JWT for OIDC")
        .arg(
            clap::Arg::with_name("keyfile")
                .help("Location of the RSA DER keypair as a file")
                .required(false)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("port")
                .short('p')
                .long("port")
                .value_name("port")
                .help("Sets the port to listen to")
                .default_value("8080"),
        )
        .arg(
            clap::Arg::with_name("bind")
                .short('b')
                .long("bind")
                .value_name("bind")
                .help("Sets the host or IP number to bind to")
                .default_value("0.0.0.0"),
        )
        .arg(
            clap::Arg::with_name("host")
                .short('h')
                .long("host")
                .value_name("host")
                .help("Full base URL of the host the service is found, like https://accounts.google.com")
                .default_value("http://localhost:8080"),
        )
        .get_matches();

    let keyfile = args
        .value_of("keyfile")
        .unwrap_or("./static/private_key.der")
        .to_owned();

    let bind_host = args.value_of("bind").unwrap();
    let bind_port = args.value_of("port").unwrap();
    let exposed_host = args.value_of("host").unwrap().to_string();

    let bind = format!("{}:{}", bind_host, bind_port);

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut user = String::from_utf8(Command::new("whoami").output().unwrap().stdout).unwrap();
    user.pop();
    println!("Mock OIDC endpoint bound to {} as user {}!", bind, user);
    //let validator = OIDCValidator::new_for_jwks(jwk_set).unwrap();
    //Start the service with some users inside
    HttpServer::new(move || {
        let rsa_keys = Secret::rsa_keypair_from_file(&keyfile)
            .expect("Cannot read RSA keypair");
        let jwk_set = discovery::create_jwk_set(&rsa_keys);

        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(web::JsonConfig::default().limit(4096)))
            .app_data(web::Data::new(AppState::new(
                &rsa_keys,
                exposed_host.clone(),
            )))
            .app_data( OIDCValidatorConfig {
                issuer: "".to_string(),
                validator: OIDCValidator::new_for_jwks(jwk_set).unwrap(),
            })
            .service(web::resource("/token").route(web::post().to(token::create_token)))
            .service(web::resource("/userinfo").route(web::get().to(userinfo::user_info)))
            .service(
                web::resource("/.well-known/openid-configuration")
                    .route(web::get().to(discovery::openid_configuration)),
            )
            .service(web::resource("/keys").route(web::get().to(discovery::keys)))
            .service(web::resource("/health").route(web::get().to(checks::check)))
    })
    .bind(bind)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_appstate() {
        let exposed_host = "http://localhost:8080".to_string();
        let rsa_keys = Secret::rsa_keypair_from_file("./static/private_key.der")
            .expect("Cannot read RSA keypair");
        let _app_state = AppState::new(&rsa_keys, exposed_host);
    }
}
