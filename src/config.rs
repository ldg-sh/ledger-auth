use std::env;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct EnvConfig {
    pub port: i32,
    pub db_url: String,
    pub admin_key: String,
    pub resend_key: String
}

impl EnvConfig {
    fn get_env(key: &str) -> String {
        env::var(key).unwrap_or_else(|_| panic!("Environment variable {} not set", key))
    }

    pub fn from_env() -> Self {
        dotenv::dotenv().ok();


        let db_url: String = Self::get_env("DATABASE_URL");
        let resend_key: String = Self::get_env("RESEND_KEY");

        EnvConfig {
            port: Self::get_env("PORT").parse().unwrap_or(8080),
            db_url,
            admin_key: Self::get_env("ADMIN_KEY"),
            resend_key
        }
    }
}

pub static CONFIG: OnceLock<EnvConfig> = OnceLock::new();

#[allow(dead_code)]
pub fn config() -> &'static EnvConfig {
    CONFIG.get().expect("Not initialized")
}
