use async_trait::async_trait;
use axum::{extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use tower_cookies::Cookies;

use crate::{Error, Result};

pub struct Token(pub String);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Token {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let cookie = parts
            .extract::<Cookies>()
            .await
            .map_err(|_| Error::Unauthorized)?;

        let token = cookie
            .get("token")
            .map(|cookie| cookie.value().to_string())
            .ok_or(Error::Unauthorized)?;

        Ok(Token(token))
    }
}
