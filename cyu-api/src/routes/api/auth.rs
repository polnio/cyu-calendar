use crate::app::App;
use crate::utils::response::api_error;
use crate::utils::Auth;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use cyu_fetcher::Fetcher;
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct LoginPayload {
    pub(super) username: String,
    pub(super) password: String,
}

#[derive(Debug, Serialize)]
struct LoginResponse {
    success: bool,
}

async fn login(
    cookies: Cookies,
    State(fetcher): State<Fetcher>,
    Json(payload): Json<LoginPayload>,
) -> Response {
    let token = match fetcher.login(payload.username, payload.password).await {
        Ok(token) => token,
        Err(_) => {
            return api_error(StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        }
    };
    let infos = match fetcher.get_infos(token.clone()).await {
        Ok(infos) => infos,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve informations",
            )
            .into_response()
        }
    };
    cookies.add(Cookie::build(("token", token)).path("/").build());
    cookies.add(Cookie::build(("id", infos.federation_id)).path("/").build());
    Json(LoginResponse { success: true }).into_response()
}

#[derive(Debug, Serialize)]
struct GetInfosResponse {
    id: String,
    name: String,
}

async fn get_infos(auth: Auth, State(fetcher): State<Fetcher>) -> Response {
    let infos = match fetcher.get_infos(auth.token).await {
        Ok(infos) => infos,
        Err(_) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve informations",
            )
            .into_response()
        }
    };

    Json(GetInfosResponse {
        id: infos.federation_id,
        name: infos.display_name,
    })
    .into_response()
}

pub fn routes() -> Router<App> {
    Router::new()
        .route("/login", post(login))
        .route("/infos", get(get_infos))
}
