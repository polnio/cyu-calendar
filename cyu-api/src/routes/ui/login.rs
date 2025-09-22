use super::{check_unauth, render_template_or_fail};
use crate::app::{App, TemplateEngine};
use crate::utils::response::ui_error;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse as _, Redirect, Response};
use axum::routing::get;
use axum::{middleware, Form};
use cyu_fetcher::Fetcher;
use serde::Deserialize;
use serde_json::Value;
use tower_cookies::{Cookie, Cookies};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub(super) struct LoginQuery {
    pub(super) redirect: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct LoginHandlePayload {
    username: String,
    password: String,
}

async fn login(State(te): State<TemplateEngine>) -> Response {
    render_template_or_fail(te, "login", None::<Value>)
}

async fn login_handle(
    cookies: Cookies,
    Query(query): Query<LoginQuery>,
    State(fetcher): State<Fetcher>,
    Form(payload): Form<LoginHandlePayload>,
) -> Response {
    let token = match fetcher.login(payload.username, payload.password).await {
        Ok(token) => token,
        Err(cyu_fetcher::errors::Error::Unauthorized) => {
            return ui_error(StatusCode::UNAUTHORIZED, "Invalid credentials".to_owned())
                .into_response()
        }
        Err(_) => {
            return ui_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to login to cyu".to_owned(),
            )
            .into_response()
        }
    };
    let infos = match fetcher.get_infos(token.clone()).await {
        Ok(infos) => infos,
        Err(_) => {
            return ui_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve infos from cyu".to_owned(),
            )
            .into_response()
        }
    };
    cookies.add(Cookie::new("token", token));
    cookies.add(Cookie::new("id", infos.federation_id));
    Redirect::to(&query.redirect.unwrap_or("/".into())).into_response()
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new()
        .route("/login", get(login).post(login_handle))
        .layer(middleware::from_fn(check_unauth))
}
