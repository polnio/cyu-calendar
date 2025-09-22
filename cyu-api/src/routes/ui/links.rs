use super::{check_auth, render_template_or_fail};
use crate::app::{App, Database, TemplateEngine};
use crate::utils::Auth;
use axum::extract::State;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct TokenRow {
    id: i64,
    token: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct LinksData {
    tokens: Vec<TokenRow>,
}

async fn links(
    State(te): State<TemplateEngine>,
    State(db): State<Database>,
    auth: Auth,
) -> impl IntoResponse {
    let tokens = sqlx::query_as!(
        TokenRow,
        "SELECT id, token FROM icstokens WHERE userid = ?",
        auth.id
    )
    .fetch_all(&*db)
    .await
    .unwrap_or_else(|err| {
        eprintln!("Failed to fetch tokens: {err}");
        Default::default()
    });

    render_template_or_fail(te, "links", Some(LinksData { tokens }))
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new()
        .route("/links", get(links))
        .layer(middleware::from_fn(check_auth))
}
