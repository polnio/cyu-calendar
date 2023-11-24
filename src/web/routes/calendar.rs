use crate::{
    app::{App, Requester},
    web::utils::auth::Token,
    Error, Result,
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
enum CalendarView {
    #[serde(rename(serialize = "agendaDay", deserialize = "day"))]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
}

/* impl Serialize for CalendarView {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            CalendarView::Day => serializer.serialize_str("agendaDay"),
            CalendarView::Week => serializer.serialize_str("week"),
            CalendarView::Month => serializer.serialize_str("month"),
        }
    }
} */

/* impl Deserialize for CalendarView {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
} */

#[derive(Deserialize)]
struct GetCalendarQuery {
    start: String,
    end: String,
    view: CalendarView,
}

#[derive(Serialize)]
struct GetCalendarRemotePayload {
    #[serde(rename = "federationIds[]")]
    id: String,
    #[serde(rename = "resType")]
    res_type: String,
    start: String,
    end: String,
    #[serde(rename = "calView")]
    view: CalendarView,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetCalendarRemoteResponseElement {
    id: String,
    start: String,
    end: String,
    all_day: bool,
    description: String,
    department: String,
    faculty: Option<String>,
    event_category: String,
    sites: Vec<String>,
    modules: Option<Vec<String>>,
}

type GetCalendarRemoteResponse = Vec<GetCalendarRemoteResponseElement>;

type GetCalendarResponse = GetCalendarRemoteResponse;

async fn get_calendar(
    Query(query): Query<GetCalendarQuery>,
    Token(token): Token,
    State(requester): State<Requester>,
    Path(id): Path<String>,
) -> Result<Json<GetCalendarResponse>> {
    let remote_payload = GetCalendarRemotePayload {
        id,
        res_type: "104".to_string(),
        start: query.start,
        end: query.end,
        view: query.view,
    };

    let response = requester
        .post("https://services-web.cyu.fr/calendar/Home/GetCalendarData")
        .form(&remote_payload)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", token)
        .send()
        .await
        .map_err(|_| Error::RemoteError)?;

    let calendar = response
        .json::<GetCalendarRemoteResponse>()
        .await
        .map_err(|_| Error::RemoteError)?;

    Ok(Json(calendar))
}

pub fn routes() -> Router<App> {
    Router::new().route("/:id", get(get_calendar))
}
