use crate::app::App;
use axum::http::{header, StatusCode, Uri};
use axum::response::IntoResponse;
use axum::routing::get;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "dist"]
struct Assets;

async fn static_file(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/');
    if path.starts_with("dist/") {
        path = &path[5..]
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new().route("/*file", get(static_file))
}
