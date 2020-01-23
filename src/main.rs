//#![feature(alloc_system)]
//extern crate alloc_system;

use actix_web::{middleware, web, App, HttpServer};
use biscuit::jws::Secret;
use clap;
use std::process::Command;

mod checks;
mod discovery;
mod token;

//AppState object is initialized for the App and passed with every request that has a parameter with the AppState as type.
pub struct AppState {
    rsa_key_pair: biscuit::jws::Secret,
}

impl AppState {
    pub fn new(private_key_location: &str) -> Self {
        Self {
            rsa_key_pair: Secret::rsa_keypair_from_file(private_key_location)
                .expect("Cannot read RSA keypair"),
        }
    }
}

/*
Profiling: http://carol-nichols.com/2015/12/09/rust-profiling-on-osx-cpu-time/
*/
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = clap::App::new("oidc-token-test-service")
        .version("0.1")
        .about("Allows to generate any valid JWT for OIDC")
        .arg(
            clap::Arg::with_name("keyfile")
                .help("Location of the RSA DER keypair as a file")
                .required(false)
                .index(1),
        )
        .arg(clap::Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("port")
                .help("Sets the port to listen to")
                .default_value("8080"))
        .arg(clap::Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("bind")
                .help("Sets the host or IP number to bind to")
                .default_value("0.0.0.0"))
        .get_matches();

    let keyfile = args
        .value_of("keyfile")
        .unwrap_or("./static/private_key.der")
        .to_owned();

    let bind = format!("{}:{}", args.value_of("bind").unwrap(), args.value_of("port").unwrap());

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut user = String::from_utf8(Command::new("whoami").output().unwrap().stdout).unwrap();
    user.pop();
    println!("Mock OIDC endpoint bound to {} as user {}!", bind, user);

    //Start the service with some users inside
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .data(web::JsonConfig::default().limit(4096))
            .data(AppState::new(&keyfile))
            .service(web::resource("/token").route(web::post().to(token::create_token)))
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
        let _app_state = AppState::new("./static/private_key.der");
    }

    #[test]
    #[should_panic(
        expected = "Cannot read RSA keypair: IOError(Os { code: 2, kind: NotFound, message: \"No such file or directory\" })"
    )]
    fn test_create_appstate_not_found_file() {
        let _app_state = AppState::new("./does_not_exist");
    }
}
