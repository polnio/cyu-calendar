use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum CalendarView {
    #[serde(rename(serialize = "agendaDay", deserialize = "day"))]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
}

pub struct GetCalendarQuery {
    pub id: String,
    pub token: String,
    pub start: String,
    pub end: String,
    pub view: CalendarView,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarResponseElement {
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
pub type GetCalendarResponse = Vec<GetCalendarResponseElement>;

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

pub async fn get_calendar(
    requester: &reqwest::Client,
    query: GetCalendarQuery,
) -> Result<GetCalendarResponse, ()> {
    let remote_payload = GetCalendarRemotePayload {
        id: query.id,
        res_type: String::from("104"),
        start: query.start,
        end: query.end,
        view: query.view,
    };

    let response = requester
        .post("https://services-web.cyu.fr/calendar/Home/GetCalendarData")
        .form(&remote_payload)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", query.token)
        .send()
        .await
        .map_err(|_| ())?;

    let calendar = response
        .json::<GetCalendarResponse>()
        .await
        .map_err(|_| ())?;

    Ok(calendar)
}
