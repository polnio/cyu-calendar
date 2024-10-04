pub mod api;
pub mod ui;

use crate::app::App;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

pub fn get() -> axum::Router<App> {
    let assets_path = format!("{}/dist", env!("CARGO_PKG_NAME"));
    axum::Router::new()
        .nest("/", ui::routes())
        .nest("/api", api::routes())
        .nest_service("/assets", ServeDir::new(assets_path))
        .layer(CookieManagerLayer::new())
}
