pub mod api;
pub mod ui;

use crate::app::App;
use tower_cookies::CookieManagerLayer;

pub fn get() -> axum::Router<App> {
    axum::Router::new()
        .nest("/", ui::routes())
        .nest("/api", api::routes())
        .layer(CookieManagerLayer::new())
}
