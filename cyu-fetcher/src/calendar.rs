use crate::{errors::Error, utils::{CyuDate, CyuDateTime}};
use chrono::NaiveDate;
use getset::Getters;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::*;
use std::error::Error as _;

pub const NB_LOCATIONS: usize = 4;
pub const LOCATIONS_COORD: [[f64; 2]; NB_LOCATIONS] = [
    [49.0350203, 2.0695627],
    [49.03899, 2.0749315],
    [49.043664, 2.0844198],
    [49.0326943, 2.0665439],
];
pub const LOCATIONS_NAME: [&str; NB_LOCATIONS] = ["PARC", "CHENES", "SAINT MARTIN", "PORT"];

#[derive(Serialize, Deserialize)]
pub enum CalendarView {
    #[serde(rename(serialize = "agendaDay", deserialize = "day"))]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
}

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ColorBy {
    EventCategory = 3,
    Subject = 6,
}

pub struct GetCalendarQuery {
    pub id: String,
    pub token: String,
    pub start: CyuDate,
    pub end: CyuDate,
    pub view: CalendarView,
    pub color_by: ColorBy,
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone)]
#[get = "pub"]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarResponseElement {
    id: String,
    start: CyuDateTime,
    end: Option<CyuDateTime>,
    all_day: bool,
    #[getset(skip)]
    #[serde(serialize_with = "serialize_description")]
    description: String,
    background_color: String,
    department: String,
    faculty: Option<String>,
    event_category: String,
    sites: Option<Vec<String>>,
    modules: Option<Vec<String>>,
}
fn parse_description(description: &str) -> String {
    static LINEBREAKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\r\n|<br />)+").unwrap());
    html_escape::decode_html_entities(LINEBREAKS_REGEX.replace_all(description, "\n").trim())
        .to_string()
}
fn serialize_description<S: Serializer>(description: &String, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&parse_description(&description))
}
impl GetCalendarResponseElement {
    pub fn coords(&self) -> Option<&[f64; 2]> {
        self.sites()
            .as_ref()
            .and_then(|sites| sites.first())
            .and_then(|site| LOCATIONS_NAME.iter().position(|name| name == site))
            .map(|location| LOCATIONS_COORD.get(location).unwrap())
    }
    pub fn description(&self) -> String {
        parse_description(&self.description)
    }
}
pub type GetCalendarResponse = Vec<GetCalendarResponseElement>;

#[derive(Serialize)]
struct GetCalendarRemotePayload {
    #[serde(rename = "federationIds[]")]
    id: String,
    #[serde(rename = "resType")]
    res_type: String,
    start: CyuDate,
    end: CyuDate,
    #[serde(rename = "calView")]
    view: CalendarView,
    #[serde(rename = "colourScheme")]
    color_by: ColorBy,
}

pub async fn get_calendar(
    requester: &reqwest::Client,
    query: GetCalendarQuery,
) -> Result<GetCalendarResponse, Error> {
    let remote_payload = GetCalendarRemotePayload {
        id: query.id,
        res_type: String::from("104"),
        start: query.start,
        end: query.end,
        view: query.view,
        color_by: query.color_by,
    };

    let response = requester
        .post("https://services-web.cyu.fr/calendar/Home/GetCalendarData")
        .form(&remote_payload)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", query.token)
        .send()
        .await
        .map_err(|err| {
            eprintln!("{}", err);
            Error::Remote
        })?;

    let calendar = response
        .json::<GetCalendarResponse>()
        .await
        .map_err(|err| {
            let source = err.source();
            if let Some(source) = source {
                if source.to_string().contains("at line 1 column 0") {
                    return Error::Unauthorized;
                }
            }
            eprintln!("{:#?}", err);
            Error::Remote
        })?;

    Ok(calendar)
}

pub struct GetLimitsQuery<'a> {
    pub id: &'a str,
    pub token: &'a str,
}

pub type GetLimitsResponse = (CyuDate, CyuDate);

pub async fn get_limits(requester: &reqwest::Client, query: GetLimitsQuery<'_>) -> Result<GetLimitsResponse, Error> {
    let page_response = requester
        .get(format!("https://services-web.cyu.fr/calendar/?CalendarViewType=Month&CalendarDate=09/29/2024 00:00:00&EntityType=Student&FederationIds={}&CalendarViewStr=month&EntityTypeAsIntegerString=104&IsValid=True&NotAllowedToBrowse=False", query.id))
        .header("Cookie", query.token)
        .send()
        .await
        .map_err(|_| Error::Remote)?;
    let page_text = page_response.text().await.map_err(|_| Error::Remote)?;

    static LIMITS_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?m)var dateExtents = \{\r\n *earliest: new Date\(([0-9]+), ([0-9]+) - 1, ([0-9]+)\),\r\n *latest: new Date\(([0-9]+), ([0-9]+) - 1, ([0-9]+)\)\r\n *\};"#)
            .unwrap()
    });

    let (_, [y1, m1, d1, y2, m2, d2]) = LIMITS_REGEX
        .captures(&page_text)
        .map(|captures| captures.extract())
        .ok_or(Error::Remote)?;

    let y1 = y1.parse().unwrap();
    let m1 = m1.parse().unwrap();
    let d1 = d1.parse().unwrap();
    let y2 = y2.parse().unwrap();
    let m2 = m2.parse().unwrap();
    let d2 = d2.parse().unwrap();

    let date1 = NaiveDate::from_ymd_opt(y1, m1, d1).ok_or(Error::Remote)?;
    let date2 = NaiveDate::from_ymd_opt(y2, m2, d2).ok_or(Error::Remote)?;
    Ok((date1.into(), date2.into()))
}

pub struct GetAllQuery {
    pub id: String,
    pub token: String,
    pub color_by: ColorBy,
}


pub async fn get_all(requester: &reqwest::Client, query: GetAllQuery) -> Result<GetCalendarResponse, Error> {
    let (start, end) = get_limits(requester, GetLimitsQuery {
            id: &query.id,
            token: &query.token
        })
        .await?;

    let events = get_calendar(requester, GetCalendarQuery {
            id: query.id,
            token: query.token,
            start,
            end,
            view: CalendarView::Month,
            color_by: query.color_by
        })
        .await?;

    Ok(events)
}
