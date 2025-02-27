use rocket::fairing::AdHoc;
use rocket::figment::value::{Map, Value};
use rocket::figment::{map, Figment};
use std::env;

/// Debug only secret for JWT encoding & decoding.
const SECRET: &'static str = "8Xui8SN4mI+7egV/9dlfYYLGQJeEx4+DwmSQLwDVXJg=";

/// js toISOString() in test suit can't handle chrono's default precision
pub const DATE_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.3fZ";

pub const TOKEN_PREFIX: &'static str = "Token ";

#[derive(Default)]
pub struct AppState {
    pub secret: Vec<u8>,
}

impl AppState {
    pub fn manage() -> AdHoc {
        AdHoc::on_ignite("Manage config", |rocket| async {
            // Rocket doesn't expose it's own secret_key, so we use our own here.
            let secret = env::var("SECRET_KEY").unwrap_or_else(|err| {
                if cfg!(debug_assertions) {
                    SECRET.to_string()
                } else {
                    panic!("No SECRET_KEY environment variable found: {:?}", err)
                }
            });

            rocket.manage(AppState {
                secret: secret.into_bytes(),
            })
        })
    }
}

/// Create rocket config from environment variables
pub fn from_env() -> Figment {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("PORT environment variable should parse to an integer");

    let database_url =
        env::var("DATABASE_URL").expect("No DATABASE_URL environment variable found");

    let database_config: Map<_, Value> = map! {
        "url" => database_url.into()
    };

    rocket::Config::figment()
        .merge(("port", port))
        .merge(("databases", map!["postgres_db_pool" => database_config]))
}
