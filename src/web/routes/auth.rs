use crate::{
    app::{App, Requester},
    web::utils::auth::Token,
    Error, Result,
};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tower_cookies::{Cookie, Cookies};

lazy_static::lazy_static! {
    static ref SECURITY_TOKEN_REGEX: Regex = Regex::new(r#"<input name="__RequestVerificationToken" type="hidden" value="([^"]+)" />"#).unwrap();
    static ref FEDERATION_ID_REGEX: Regex = Regex::new(r#"var federationIdStr = '(.*?)';"#).unwrap();
}

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
    State(requester): State<Requester>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>> {
    let page_response = requester
        .get("https://services-web.cyu.fr/calendar/LdapLogin")
        .send()
        .await
        .map_err(|_| Error::RemoteError)?;
    let page_cookie = page_response
        .headers()
        .get("set-cookie")
        .map(|cookie| cookie.clone())
        .ok_or(Error::RemoteError)?;
    let plain_text = page_response.text().await.map_err(|_| Error::RemoteError)?;
    let token = SECURITY_TOKEN_REGEX
        .captures(&plain_text)
        .and_then(|captures| captures.get(1))
        .map(|token| token.as_str().to_string())
        .ok_or(Error::RemoteError)?;

    let mut remote_payload = HashMap::new();
    remote_payload.insert("Name", payload.username);
    remote_payload.insert("Password", payload.password);
    remote_payload.insert("__RequestVerificationToken", token);

    let login_response = requester
        .post("https://services-web.cyu.fr/calendar/LdapLogin/Logon")
        .form(&remote_payload)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", page_cookie)
        .send()
        .await
        .map_err(|_| Error::RemoteError)?;
    let login_cookie = login_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|cookie| cookie.to_str().unwrap_or_default().to_string())
        .join(";");

    cookies.add(Cookie::new("token", login_cookie));

    Ok(Json(LoginResponse { success: true }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoteInfosResponse {
    federation_id: String,
    display_name: String,
}

#[derive(Debug, Serialize)]
struct GetInfosResponse {
    id: String,
    name: String,
}

async fn get_infos(
    Token(token): Token,
    State(requester): State<Requester>,
) -> Result<Json<GetInfosResponse>> {
    let federation_id_response = requester
        .get("https://services-web.cyu.fr/calendar")
        .header("Cookie", token.clone())
        .send()
        .await
        .map_err(|_| Error::RemoteError)?;

    let plain_text = federation_id_response
        .text()
        .await
        .map_err(|_| Error::RemoteError)?;

    let federation_id = FEDERATION_ID_REGEX
        .captures(&plain_text)
        .and_then(|captures| captures.get(1))
        .map(|federation_id| federation_id.as_str().to_string())
        .ok_or(Error::RemoteError)?;

    let name_response = requester
        .post("https://services-web.cyu.fr/calendar/Home/LoadDisplayNames")
        .form(&json!({
            "federationIds[]": federation_id,
            "resType": 104
        }))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", token)
        .send()
        .await
        .map_err(|_| Error::RemoteError)?;

    let infos = name_response
        .json::<Vec<RemoteInfosResponse>>()
        .await
        .map_err(|_| Error::RemoteError)
        .and_then(|infos| infos.into_iter().next().ok_or(Error::RemoteError))?;

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
