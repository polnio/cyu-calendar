use axum::extract::FromRef;

pub type Requester = reqwest::Client;

#[derive(Clone)]
pub struct App {
    requester: Requester,
}

impl App {
    pub fn new() -> Self {
        Self {
            requester: Requester::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        }
    }
}

impl FromRef<App> for Requester {
    fn from_ref(app: &App) -> Self {
        app.requester.clone()
    }
}
