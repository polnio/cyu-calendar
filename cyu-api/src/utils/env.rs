#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    pub ics_auth_key: String,
}

impl Env {
    pub fn load() -> Self {
        if let Err(err) = dotenv::dotenv() {
            eprintln!("Failed to load .env: {err}");
        }
        let ics_auth_key = match std::env::var("ICS_AUTH_KEY") {
            Ok(ics_auth_key) => ics_auth_key,
            Err(err) => {
                eprintln!("Failed to load .env: {err}");
                std::process::exit(1);
            }
        };
        Self { ics_auth_key }
    }
}
