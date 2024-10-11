use crate::{Error, Result};
use crate::utils::{ics, Auth};
use crate::app::{App, Encrypter};
use axum::{Json, Router};
use axum::routing::get;
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};
use axum::extract::{Query, State};
use cyu_fetcher::{calendar::ColorBy, utils::CyuDate, Fetcher};
use serde::Deserialize;

use super::auth::LoginPayload;

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

async fn get_ics_token(
    State(encrypter): State<Encrypter>,
    Json(payload): Json<LoginPayload>,
) -> Response {
    let Ok(token) = ics::encrypt_creds(&encrypter, &payload.username, &payload.password) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    };
    token.into_response()
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct GetIcsQuery {
    token: String
}

async fn get_ics(
    State(encrypter): State<Encrypter>,
    State(fetcher): State<Fetcher>,
    // auth: Auth,
    Query(query): Query<GetIcsQuery>,
) -> Response {
    let Ok((username, password)) = ics::decrypt_creds(&encrypter, &query.token) else {
        return (StatusCode::UNAUTHORIZED, "").into_response();
    };
    let Ok(token) = fetcher.login(username, password).await else {
        return (StatusCode::UNAUTHORIZED, "").into_response();
    };
    let Ok(infos) = fetcher.get_infos(token.clone()).await else {
        return (StatusCode::UNAUTHORIZED, "").into_response();
    };
    let auth = Auth {
        id: infos.federation_id,
        token
    };
    let Ok(calendar) = ics::generate(&fetcher, auth).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    };
    ([(header::CONTENT_TYPE, "text/calendar")], calendar).into_response()
}

pub fn routes() -> Router<App> {
    Router::new()
        .route("/", get(get_calendar))
        .route("/ics", get(get_ics))
        .route("/ics-token", get(get_ics_token))
}
