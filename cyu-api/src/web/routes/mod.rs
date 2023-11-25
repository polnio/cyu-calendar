use tower_cookies::CookieManagerLayer;

use crate::app::App;

pub mod auth;
pub mod calendar;

pub fn get() -> axum::Router<App> {
    axum::Router::new()
        .nest("/auth", auth::routes())
        .nest("/calendar", calendar::routes())
        .layer(CookieManagerLayer::new())
}
