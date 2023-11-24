use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
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
        println!("{:?}", self);

        (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response()
    }
}
