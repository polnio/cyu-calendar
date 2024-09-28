use crate::{app::App, web::utils::auth::Token, Error, Result};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use cyu_fetcher::Fetcher;
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
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
    cookies.add(Cookie::new("token", token));
    Ok(Json(LoginResponse { success: true }))
}

#[derive(Debug, Serialize)]
struct GetInfosResponse {
    id: String,
    name: String,
}

async fn get_infos(
    Token(token): Token,
    State(fetcher): State<Fetcher>,
) -> Result<Json<GetInfosResponse>> {
    let infos = fetcher
        .get_infos(token)
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
