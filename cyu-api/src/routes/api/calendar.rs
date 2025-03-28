use crate::app::{App, Database, Encrypter};
use crate::utils::body::Body;
use crate::utils::response::api_error;
use crate::utils::{ics, Auth};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use cyu_fetcher::{calendar::ColorBy, utils::CyuDate, Fetcher};
use serde::Deserialize;
use std::ops::Deref;

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
) -> Response {
    let calendar = fetcher
        .get_calendar(cyu_fetcher::calendar::GetCalendarQuery {
            id: auth.id,
            token: auth.token,
            start: query.start,
            end: query.end,
            view: query.view,
            color_by: ColorBy::EventCategory,
        })
        .await;

    match calendar {
        Ok(calendar) => Json(calendar).into_response(),
        Err(_) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve calendar from cyu",
        )
        .into_response(),
    }
}

async fn post_ics_token(
    State(encrypter): State<Encrypter>,
    State(db): State<Database>,
    auth: Auth,
    payload: Body<LoginPayload>,
) -> Response {
    let Ok(token) = encrypter.encrypt(payload.deref()) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    };
    let tfp = &token[..12];
    let result = sqlx::query!(
        "INSERT INTO icstokens (userid,token) VALUES (?, ?)",
        auth.id,
        tfp
    )
    .execute(&*db)
    .await;
    if let Err(_) = result {
        return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    }
    token.into_response()
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct DeleteIcsQuery {
    token_id: i64,
}

async fn delete_ics_token(
    State(db): State<Database>,
    Query(query): Query<DeleteIcsQuery>,
    auth: Auth,
) -> Response {
    let result = sqlx::query!(
        "DELETE FROM icstokens WHERE userid = ? AND id = ?",
        auth.id,
        query.token_id
    )
    .execute(&*db)
    .await;
    if let Err(_) = result {
        return (StatusCode::INTERNAL_SERVER_ERROR, "").into_response();
    }
    "".into_response()
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct GetIcsQuery {
    token: String,
}

async fn get_ics(
    State(encrypter): State<Encrypter>,
    State(fetcher): State<Fetcher>,
    // auth: Auth,
    Query(query): Query<GetIcsQuery>,
) -> Response {
    let Ok((username, password)) = encrypter.decrypt(&query.token) else {
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
        token,
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
        .route("/ics-token", post(post_ics_token).delete(delete_ics_token))
}
