use axum::http::uri::PathAndQuery;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use serde_json::{json, Value};

type ApiResponse = (StatusCode, Json<Value>);
type UiResponse = (StatusCode, String);

pub fn api_error(status_code: StatusCode, error: &str) -> ApiResponse {
    (
        status_code,
        Json(json!({
            "success": false,
            "message": error
        })),
    )
}

pub fn ui_error(status_code: StatusCode, error: String) -> UiResponse {
    (status_code, error)
}

pub fn redirect_to_login(current_uri: &Uri) -> impl IntoResponse {
    let redirect = current_uri
        .path_and_query()
        .map_or("/", PathAndQuery::as_str);
    Redirect::to(&format!("/login?redirect={}", redirect))
}

pub trait AnyhowExt {
    fn into_api_response(&self, status_code: StatusCode) -> ApiResponse;
    fn into_ui_response(&self, status_code: StatusCode) -> UiResponse;
}

impl AnyhowExt for anyhow::Error {
    fn into_api_response(&self, status_code: StatusCode) -> (StatusCode, Json<Value>) {
        api_error(status_code, &self.to_string())
    }
    fn into_ui_response(&self, status_code: StatusCode) -> (StatusCode, String) {
        ui_error(status_code, self.to_string())
    }
}
