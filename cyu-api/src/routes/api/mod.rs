pub mod auth;
pub mod calendar;

use crate::app::App;

pub fn routes() -> axum::Router<App> {
    axum::Router::new()
        .nest("/auth", auth::routes())
        .nest("/calendar", calendar::routes())
}
