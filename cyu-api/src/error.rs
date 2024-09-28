use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use handlebars::RenderError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    RenderTemplate(RenderError),

    RemoteError,
    BadCredentials,

    Unauthorized,
    BadRequest,
    InternalError,
    NotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        #[cfg(debug_assertions)]
        eprintln!("{:?}", self);

        (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response()
    }
}
