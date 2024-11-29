use super::{check_auth, default_date_for_view, render_template_or_fail};
use crate::app::{App, TemplateEngine};
use crate::routes::ui::set_uri;
use crate::utils::response::{redirect_to_login, ui_error};
use crate::utils::Auth;
use axum::extract::{OriginalUri, Query, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse as _, Redirect, Response};
use axum::routing::get;
use chrono::{Days, Months};
use cyu_fetcher::utils::CyuDate;
use cyu_fetcher::Fetcher;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};

#[derive(Serialize)]
struct HomeData {
    calendar: cyu_fetcher::calendar::GetCalendarResponse,
    previous_page: String,
    next_page: String,
}
#[derive(Default, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub(super) enum HomeQueryView {
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
            Self::Week => cyu_fetcher::calendar::CalendarView::Week,
        }
    }
}
#[derive(Deserialize)]
struct HomeQuery {
    date: Option<CyuDate>,
    view: Option<HomeQueryView>,
}

async fn home(
    auth: Auth,
    cookies: Cookies,
    OriginalUri(uri): OriginalUri,
    Query(query): Query<HomeQuery>,
    State(te): State<TemplateEngine>,
    State(fetcher): State<Fetcher>,
) -> Response {
    let uri_string = uri.to_string();
    let (start, view) = match (query.date, query.view) {
        (Some(date), Some(view)) => (date, view),
        (Some(date), None) => {
            let view = HomeQueryView::default();
            // return Ok(Redirect::to(&format!("{uri}&view={view}")).into_response())
            println!("{}, {}", uri, set_uri(&uri_string, &date, &view));
            return Redirect::to(&set_uri(&uri_string, &date, &view)).into_response();
        }
        (None, Some(view)) => {
            let date = default_date_for_view(&view);
            // return Ok(Redirect::to(&format!("{uri}&date={date}")).into_response())
            return Redirect::to(&set_uri(&uri_string, &date, &view)).into_response();
        }
        (None, None) => {
            // let sep = if uri.query().is_some() {'&'} else {'?'};
            let view = HomeQueryView::default();
            let date = default_date_for_view(&view);
            // return Ok(Redirect::to(&format!("{uri}{sep}date={date}&view={view}")).into_response())
            return Redirect::to(&set_uri(&uri_string, &date, &view)).into_response();
        }
    };

    let (previous, next): (CyuDate, CyuDate) = match &view {
        HomeQueryView::Month => {
            let previous = start
                .checked_sub_months(Months::new(1))
                .unwrap_or_else(|| *start.clone())
                .into();
            let next = start
                .checked_add_months(Months::new(1))
                .unwrap_or_else(|| *start.clone())
                .into();
            (previous, next)
        }
        HomeQueryView::Week => {
            let previous = start
                .checked_sub_days(Days::new(7))
                .unwrap_or_else(|| *start.clone())
                .into();
            let next = start
                .checked_add_days(Days::new(7))
                .unwrap_or_else(|| *start.clone())
                .into();
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
        .await;

    let calendar = match calendar {
        Ok(calendar) => calendar,
        Err(cyu_fetcher::Error::Unauthorized) => {
            cookies.remove(Cookie::from("token"));
            cookies.remove(Cookie::from("id"));
            return redirect_to_login(&uri).into_response();
        }
        Err(_) => {
            return ui_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve calendar from cyu".to_owned(),
            )
            .into_response()
        }
    };

    render_template_or_fail(
        te,
        "home",
        Some(HomeData {
            calendar,
            previous_page,
            next_page,
        }),
    )
}

pub fn routes() -> axum::Router<App> {
    axum::Router::new()
        .route("/", get(home))
        .layer(middleware::from_fn(check_auth))
}
