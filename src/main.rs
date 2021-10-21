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
    exposed_host: String,
}

impl AppState {
    pub fn new(private_key_location: &str, exposed_host: String) -> Self {
        Self {
            rsa_key_pair: Secret::rsa_keypair_from_file(private_key_location)
                .expect("Cannot read RSA keypair"),
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
                .short("p")
                .long("port")
                .value_name("port")
                .help("Sets the port to listen to")
                .default_value("8080"),
        )
        .arg(
            clap::Arg::with_name("bind")
                .short("b")
                .long("bind")
                .value_name("bind")
                .help("Sets the host or IP number to bind to")
                .default_value("0.0.0.0"),
        )
        .arg(
            clap::Arg::with_name("host")
                .short("h")
                .long("host")
                .value_name("exposed_host")
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

    //Start the service with some users inside
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(web::JsonConfig::default().limit(4096)))
            .app_data(web::Data::new(AppState::new(
                &keyfile,
                exposed_host.clone(),
            )))
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
        let exposed_host = "http://localhost:8080".to_string();
        let _app_state = AppState::new("./static/private_key.der", exposed_host);
    }

    #[test]
    #[should_panic(
        expected = "Cannot read RSA keypair: IOError(Os { code: 2, kind: NotFound, message: \"No such file or directory\" })"
    )]
    fn test_create_appstate_not_found_file() {
        let exposed_host = "http://localhost:8080".to_string();
        let _app_state = AppState::new("./does_not_exist", exposed_host);
    }
}
