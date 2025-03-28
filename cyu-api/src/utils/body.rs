use axum::extract::{FromRequest, Json, Request};
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::Form;
use derive_more::Deref;
use serde::Deserialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deref)]
pub struct Body<T>(T);

impl<T, S> FromRequest<S> for Body<T>
where
    S: Send + Sync,
    T: for<'de> Deserialize<'de>,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/json");
        let payload = match content_type {
            "application/json" => {
                let body = Json::from_request(req, state)
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;

                body.0
            }
            "application/x-www-form-urlencoded" => {
                let body = Form::from_request(req, state)
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
                body.0
            }
            _ => {
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        Ok(Body(payload))
    }
}
