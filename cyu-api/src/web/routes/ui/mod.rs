use std::collections::HashMap;

use axum::extract::{OriginalUri, Query, Request};
use axum::http::uri::PathAndQuery;
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Form;
use axum::{extract::State, routing::get};
use chrono::{Datelike as _, Days, Months, Weekday};
use cyu_fetcher::calendar::GetCalendarQuery;
use cyu_fetcher::utils::CyuDate;
use cyu_fetcher::Fetcher;
use derive_more::derive::Display;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use crate::web::utils::auth::get_auth_from_cookies;
use crate::web::utils::Auth;
use crate::Error;
use crate::app::{App, TemplateEngine};

#[derive(Debug, Deserialize)]
struct LoginQuery {
    redirect: Option<String>
}

pub fn render_template(te: TemplateEngine, name: &str, data: Option<impl Serialize>) -> Result<impl IntoResponse, Error> {
    let output = if let Some(data) = data {
        te.render(name, &data)
    } else {
        te.render(name, &json!({}))
    };

    Ok(Html(output.map_err(Error::from)?))
}

async fn check_auth(
    OriginalUri(uri): OriginalUri,
    cookies: Cookies,
    request: Request,
    next: Next
) -> Response {
    if get_auth_from_cookies(&cookies).is_none() {
        let redirect = uri.path_and_query().map_or("/", PathAndQuery::as_str);
        return Redirect::to(&format!("/login?redirect={}", redirect)).into_response()
    }
    next.run(request).await
}

async fn check_unauth(
    Query(query): Query<LoginQuery>,
    cookies: Cookies,
    request: Request,
    next: Next
) -> Response {
    if get_auth_from_cookies(&cookies).is_some() {
        return Redirect::to(&query.redirect.unwrap_or("/".into())).into_response()
    }
    next.run(request).await
}

#[derive(Serialize)]
struct HomeData {
    calendar: cyu_fetcher::calendar::GetCalendarResponse,
    previous_page: String,
    next_page: String,
}
#[derive(Default, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
enum HomeQueryView {
    #[display("month")]
    Month,
    #[default]
    #[display("week")]
    Week,
}
impl Into<cyu_fetcher::calendar::CalendarView> for HomeQueryView {
    fn into(self) -> cyu_fetcher::calendar::CalendarView {
        match self {
            Self::Month => cyu_fetcher::calendar::CalendarView::Month,
            Self::Week => cyu_fetcher::calendar::CalendarView::Week
        }
    }
}
#[derive(Deserialize)]
struct HomeQuery {
    date: Option<CyuDate>,
    view: Option<HomeQueryView>
}

fn default_date_for_view(view: &HomeQueryView) -> CyuDate {
    let today = CyuDate::today();
    match view {
        HomeQueryView::Month => today.with_day(1).unwrap().into(),
        HomeQueryView::Week => match today.weekday() {
            Weekday::Sat => today.checked_add_days(Days::new(2)).unwrap_or_else(|| *today.clone()).into(),
            Weekday::Sun => today.checked_add_days(Days::new(1)).unwrap_or_else(|| *today.clone()).into(),
            weekday => today.checked_sub_days(Days::new(weekday.num_days_from_monday().into())).unwrap().into()
        }
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

async fn home(
    auth: Auth,
    OriginalUri(uri): OriginalUri,
    Query(query): Query<HomeQuery>,
    State(te): State<TemplateEngine>,
    State(fetcher): State<Fetcher>
) -> Result<Response, Error> {
    let uri_string = uri.to_string();
    let (start, view) = match (query.date, query.view) {
        (Some(date), Some(view)) => (date, view),
        (Some(date), None) => {
            let view = HomeQueryView::default();
            // return Ok(Redirect::to(&format!("{uri}&view={view}")).into_response())
            println!("{}, {}", uri, set_uri(&uri_string, &date, &view));
            return Ok(Redirect::to(&set_uri(&uri_string, &date, &view)).into_response())
        }
        (None, Some(view)) => {
            let date = default_date_for_view(&view);
            // return Ok(Redirect::to(&format!("{uri}&date={date}")).into_response())
            return Ok(Redirect::to(&set_uri(&uri_string, &date, &view)).into_response())
        }
        (None, None) => {
            // let sep = if uri.query().is_some() {'&'} else {'?'};
            let view = HomeQueryView::default();
            let date = default_date_for_view(&view);
            // return Ok(Redirect::to(&format!("{uri}{sep}date={date}&view={view}")).into_response())
            return Ok(Redirect::to(&set_uri(&uri_string, &date, &view)).into_response())
        }
    };

    let (previous, next): (CyuDate, CyuDate) = match &view {
        HomeQueryView::Month => {
            let previous = start.checked_sub_months(Months::new(1)).unwrap_or_else(|| *start.clone()).into();
            let next = start.checked_add_months(Months::new(1)).unwrap_or_else(|| *start.clone()).into();
            (previous, next)
        }
        HomeQueryView::Week => {
            let previous = start.checked_sub_days(Days::new(7)).unwrap_or_else(|| *start.clone()).into();
            let next = start.checked_add_days(Days::new(7)).unwrap_or_else(|| *start.clone()).into();
            (previous, next)
        }
    };

    let previous_page = set_uri(&uri_string, &previous, &view);
    let next_page = set_uri(&uri_string, &next, &view);

    let calendar = fetcher
        .get_all_calendar(cyu_fetcher::calendar::GetAllQuery {
            id: auth.id,
            token: auth.token,
            color_by: cyu_fetcher::calendar::ColorBy::EventCategory,
        })
        .await
        .map_err(|_| Error::RemoteError)?;

    render_template(te, "home", Some(HomeData { calendar, previous_page, next_page })).map(IntoResponse::into_response)
}

async fn login(
    State(te): State<TemplateEngine>
) -> Result<impl IntoResponse, Error> {
    // Ok(Html(te.render("login", &json!({})).map_err(Error::from)?))
    render_template(te, "login", None::<Value>)
}

#[derive(Debug, Deserialize)]
struct LoginHandlePayload {
    username: String,
    password: String,
}

async fn login_handle(
    cookies: Cookies,
    Query(query): Query<LoginQuery>,
    State(fetcher): State<Fetcher>,
    Form(payload): Form<LoginHandlePayload>
) -> Result<impl IntoResponse, Error> {
    let token = match fetcher.login(payload.username, payload.password).await {
        Ok(token) => token,
        Err(cyu_fetcher::errors::Error::Unauthorized) => return Err(Error::BadCredentials),
        Err(_) => return Err(Error::RemoteError)
    };
    let infos = match fetcher.get_infos(token.clone()).await {
        Ok(infos) => infos,
        Err(cyu_fetcher::errors::Error::Unauthorized) => return Err(Error::BadCredentials),
        Err(_) => return Err(Error::RemoteError)
    };
    cookies.add(Cookie::new("token", token));
    cookies.add(Cookie::new("id", infos.federation_id));
    Ok(Redirect::to(&query.redirect.unwrap_or("/".into())))
}

pub fn routes() -> axum::Router<App> {
    let authed_routes = axum::Router::new()
        .route("/", get(home))
        .layer(middleware::from_fn(check_auth));

    let unauthed_routes = axum::Router::new()
        .route("/login", get(login).post(login_handle))
        .layer(middleware::from_fn(check_unauth));

    axum::Router::new()
        .merge(authed_routes)
        .merge(unauthed_routes)
}
