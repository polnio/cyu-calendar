mod home;
mod login;

use crate::app::{App, TemplateEngine};
use crate::utils::auth::get_auth_from_cookies;
use crate::utils::response::{redirect_to_login, AnyhowExt as _};
use anyhow::Context;
use axum::extract::{OriginalUri, Query, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};
use chrono::{Datelike as _, Days, Weekday};
use cyu_fetcher::utils::CyuDate;
use home::HomeQueryView;
use itertools::Itertools;
use login::LoginQuery;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use tower_cookies::Cookies;

pub fn render_template(
    te: TemplateEngine,
    name: &str,
    data: Option<impl Serialize>,
) -> Result<impl IntoResponse, handlebars::RenderError> {
    let output = if let Some(data) = data {
        te.render(name, &data)
    } else {
        te.render(name, &json!({}))
    };

    output.map(Html)
}

pub fn render_template_or_fail(
    te: TemplateEngine,
    name: &str,
    data: Option<impl Serialize>,
) -> Response {
    match render_template(te, name, data)
        .with_context(|| format!("Failed to render template '{name}'"))
    {
        Ok(result) => result.into_response(),
        Err(err) => err
            .into_api_response(StatusCode::INTERNAL_SERVER_ERROR)
            .into_response(),
    }
}

async fn check_auth(
    OriginalUri(uri): OriginalUri,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Response {
    if get_auth_from_cookies(&cookies).is_none() {
        redirect_to_login(&uri).into_response()
    } else {
        next.run(request).await
    }
}

async fn check_unauth(
    Query(query): Query<LoginQuery>,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Response {
    if get_auth_from_cookies(&cookies).is_some() {
        return Redirect::to(&query.redirect.unwrap_or("/".into())).into_response();
    }
    next.run(request).await
}

fn default_date_for_view(view: &HomeQueryView) -> CyuDate {
    let today = CyuDate::today();
    match view {
        HomeQueryView::Month => today.with_day(1).unwrap().into(),
        HomeQueryView::Week => match today.weekday() {
            Weekday::Sat => today
                .checked_add_days(Days::new(2))
                .unwrap_or_else(|| *today.clone())
                .into(),
            Weekday::Sun => today
                .checked_add_days(Days::new(1))
                .unwrap_or_else(|| *today.clone())
                .into(),
            weekday => today
                .checked_sub_days(Days::new(weekday.num_days_from_monday().into()))
                .unwrap()
                .into(),
        },
    }
}

fn set_uri(uri: &str, date: &CyuDate, view: &HomeQueryView) -> String {
    let Some((path, query)) = uri.split_once('?') else {
        return format!("{uri}?date={date}&view={view}");
    };

    let date = date.to_string();
    let view = view.to_string();

    let mut parts = query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .collect::<HashMap<_, _>>();

    parts.insert("date", &date);
    parts.insert("view", &view);

    let new_query = parts
        .into_iter()
        .map(|(key, value)| format!("{key}={value}"))
        .intersperse('&'.into())
        .collect::<String>();

    format!("{path}?{new_query}")
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new()
        .merge(home::routes())
        .merge(login::routes())
}
