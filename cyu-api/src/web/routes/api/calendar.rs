use crate::{app::App, web::utils::auth::Token, Error, Result};
use axum::{
    extract::{Path, Query, State}, http::header, response::IntoResponse, routing::get, Json, Router
};
use cyu_fetcher::{calendar::ColorBy, Fetcher};
use icalendar::Component;
use serde::Deserialize;

#[derive(Deserialize)]
struct GetCalendarQuery {
    start: String,
    end: String,
    view: cyu_fetcher::calendar::CalendarView,
}

async fn get_calendar(
    Query(query): Query<GetCalendarQuery>,
    Token(token): Token,
    State(fetcher): State<Fetcher>,
    Path(id): Path<String>,
) -> Result<Json<cyu_fetcher::calendar::GetCalendarResponse>> {
    let calendar = fetcher
        .get_calendar(cyu_fetcher::calendar::GetCalendarQuery {
            id,
            token,
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
    Token(token): Token,
    Path(id): Path<String>,
) -> Result<impl IntoResponse> {
    let (start, end) = fetcher.get_calendar_limits(cyu_fetcher::calendar::GetLimitsQuery { id: &id, token: &token })
        .await
        .map_err(|_| Error::RemoteError)?;

    let events = fetcher
        .get_calendar(cyu_fetcher::calendar::GetCalendarQuery {
            id,
            token,
            start: start.to_string(),
            end: end.to_string(),
            view: cyu_fetcher::calendar::CalendarView::Month,
            color_by: ColorBy::EventCategory,
        })
        .await
        .map_err(|_| Error::RemoteError)?;

    let events = events
        .into_iter()
        .map(|event| icalendar::Event::new()
             .description(&event.description())
             .done()
        );

    let mut calendar = icalendar::Calendar::from_iter(events);
    let calendar = calendar.name("CYU Calendar");

    Ok(([(header::CONTENT_TYPE, "text/calendar")], calendar.to_string()))
}

pub fn routes() -> Router<App> {
    Router::new()
        .route("/:id", get(get_calendar))
        .route("/:id/ics", get(ics))
}
