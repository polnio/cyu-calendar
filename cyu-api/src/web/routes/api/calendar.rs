use crate::{app::App, web::utils::auth::Token, Error, Result};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use cyu_fetcher::{calendar::ColorBy, Fetcher};
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

pub fn routes() -> Router<App> {
    Router::new().route("/:id", get(get_calendar))
}
