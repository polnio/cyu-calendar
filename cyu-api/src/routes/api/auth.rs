use crate::{Error, Result};
use crate::utils::Auth;
use crate::app::App;
use axum::{Json, Router};
use axum::routing::{get, post};
use axum::extract::State;
use cyu_fetcher::Fetcher;
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

#[derive(Debug, Deserialize)]
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
) -> Result<Json<LoginResponse>> {
    let token = fetcher
        .login(payload.username, payload.password)
        .await
        .map_err(|_| Error::RemoteError)?;
    let infos = fetcher
        .get_infos(token.clone())
        .await
        .map_err(|_| Error::RemoteError)?;
    cookies.add(Cookie::build(("token", token)).path("/").build());
    cookies.add(Cookie::build(("id", infos.federation_id)).path("/").build());
    Ok(Json(LoginResponse { success: true }))
}

#[derive(Debug, Serialize)]
struct GetInfosResponse {
    id: String,
    name: String,
}

async fn get_infos(
    auth: Auth,
    State(fetcher): State<Fetcher>,
) -> Result<Json<GetInfosResponse>> {
    let infos = fetcher
        .get_infos(auth.token)
        .await
        .map_err(|_| Error::RemoteError)?;

    Ok(Json(GetInfosResponse {
        id: infos.federation_id,
        name: infos.display_name,
    }))
}

pub fn routes() -> Router<App> {
    Router::new()
        .route("/login", post(login))
        .route("/infos", get(get_infos))
}
