use anyhow::{Context as _, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    pub ics_auth_key: String,
    pub database_url: String,
}

macro_rules! load_env {
    ($name:ident) => {
        std::env::var(stringify!($name)).context(concat!(stringify!($name), " is not set"))?
    };
}

impl Env {
    pub fn load() -> Result<Self> {
        if let Err(err) = dotenv::dotenv() {
            eprintln!("Warning: Failed to load .env: {err}");
        }
        Ok(Self {
            ics_auth_key: load_env!(ICS_AUTH_KEY),
            database_url: load_env!(DATABASE_URL),
        })
    }
}
