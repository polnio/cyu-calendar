use anyhow::Result;
use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use tower_cookies::Cookies;

pub struct Auth {
    pub token: String,
    pub id: String,
}

pub fn get_auth_from_cookies(cookies: &Cookies) -> Option<Auth> {
    let token = cookies.get("token")?.value().to_owned();
    let id = cookies.get("id")?.value().to_owned();
    Some(Auth { token, id })
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Auth {
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, ()> {
        let cookies = parts.extract::<Cookies>().await.map_err(|_| ())?;

        get_auth_from_cookies(&cookies).ok_or(())
    }
}
