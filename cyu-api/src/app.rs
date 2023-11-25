use axum::extract::FromRef;
use cyu_fetcher::Fetcher;

#[derive(Clone)]
pub struct App {
    requester: Fetcher,
}

impl App {
    pub fn new() -> Self {
        Self {
            requester: Fetcher::new(),
        }
    }
}

impl FromRef<App> for Fetcher {
    fn from_ref(app: &App) -> Self {
        app.requester.clone()
    }
}
