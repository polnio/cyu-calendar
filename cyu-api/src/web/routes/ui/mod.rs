use axum::response::Html;
use axum::{extract::State, routing::get};
use serde_json::json;
use crate::Error;
use crate::app::{App, TemplateEngine};

async fn home(
    State(te): State<TemplateEngine>
) -> Result<Html<String>, Error> {
    Ok(Html(te.render("home", &json!({})).map_err(Error::from)?))
    // Ok(Html(te.render("login", &json!({})).map_err(Error::from)?))
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new().route("/", get(home))
}
