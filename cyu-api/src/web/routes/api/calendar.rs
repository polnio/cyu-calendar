use std::{borrow::Cow, fmt::format, ops::Deref};

use crate::{app::App, web::utils::Auth, Error, Result};
use axum::{
    extract::{Query, State}, http::header, response::IntoResponse, routing::get, Json, Router
};
use cyu_fetcher::{calendar::ColorBy, utils::CyuDate, Fetcher};
use icalendar::{Component, EventLike};
use serde::Deserialize;

#[derive(Deserialize)]
struct GetCalendarQuery {
    start: CyuDate,
    end: CyuDate,
    view: cyu_fetcher::calendar::CalendarView,
}

async fn get_calendar(
    Query(query): Query<GetCalendarQuery>,
    auth: Auth,
    State(fetcher): State<Fetcher>,
) -> Result<Json<cyu_fetcher::calendar::GetCalendarResponse>> {
    let calendar = fetcher
        .get_calendar(cyu_fetcher::calendar::GetCalendarQuery {
            id: auth.id,
            token: auth.token,
            start: query.start,
            end: query.end,
            view: query.view,
            color_by: ColorBy::EventCategory,
        })
        .await
        .map_err(|_| Error::RemoteError)?;

    Ok(Json(calendar))
}

async fn ics(
    State(fetcher): State<Fetcher>,
    auth: Auth,
) -> Result<impl IntoResponse> {
    let events = fetcher
        .get_all_calendar(cyu_fetcher::calendar::GetAllQuery {
            id: auth.id,
            token: auth.token,
            color_by: ColorBy::EventCategory
        })
        .await
        .map_err(|_| Error::RemoteError)?;

    let events = events
        .into_iter()
        .filter_map(|event| {
            let mut ievent = icalendar::Event::new();

            let description = event.description();

            ievent
                .uid(&format!("{}@cyu-calendar", event.id()))
                .description(&description);

            match (event.all_day(), event.end()) {
                (true, _) => ievent.all_day((**event.start()).into()),
                (false, Some(end)) => ievent
                    .starts::<chrono::NaiveDateTime>(**event.start())
                    .ends::<chrono::NaiveDateTime>(**end),
                (false, None) => return None,
            };

            let category = event.event_category();
            let title: Cow<str> = match category.as_str() {
                // "CM" => format!("CM {}", event.description().split('\n').rev().nth(2).unwrap_or_default()).into(),
                // "TD" => description.split('\n').rev().nth(2).unwrap_or_default().into(),
                "CM" | "TD" => format!("{} {}", category, description.split('\n').rev().nth(2).unwrap_or_default().replace(category, "")).into(),
                cat => cat.into(),
            };
            ievent.summary(&title);

            Some(ievent.done())
        });

    let mut calendar = icalendar::Calendar::from_iter(events);
    let calendar = calendar.name("CYU Calendar");

    Ok(([(header::CONTENT_TYPE, "text/calendar")], calendar.to_string().replace("\\N", "\\n")))
}

pub fn routes() -> Router<App> {
    Router::new()
        .route("/", get(get_calendar))
        .route("/ics", get(ics))
}
