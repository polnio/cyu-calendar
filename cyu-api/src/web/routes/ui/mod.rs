use axum::routing::get;
use crate::app::App;

async fn home() -> &'static str {
    "Hello, World!"
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new().route("/", get(home))
}
