use crate::errors::Error;
use chrono::NaiveDate;
use getset::Getters;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
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

mod serde_date_time {
    use chrono::NaiveDateTime;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let date = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
            .map_err(|_| Error::custom("Invalid date"))?;
        Ok(date)
    }

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&date.format("%Y-%m-%dT%H:%M:%S").to_string())
    }
}

mod serde_option_date_time {
    use chrono::NaiveDateTime;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        match s {
            Some(s) => Ok(Some(
                chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
                    .map_err(|_| Error::custom("Invalid date"))?,
            )),
            None => Ok(None),
        }
    }

    pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => serializer.serialize_str(&date.format("%Y-%m-%dT%H:%M:%S").to_string()),
            None => serializer.serialize_none(),
        }
    }
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
    pub start: String,
    pub end: String,
    pub view: CalendarView,
    pub color_by: ColorBy,
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone)]
#[get = "pub"]
#[serde(rename_all = "camelCase")]
pub struct GetCalendarResponseElement {
    id: String,
    // start: String,
    // end: String,
    #[serde(with = "serde_date_time")]
    start: chrono::NaiveDateTime,
    #[serde(with = "serde_option_date_time")]
    end: Option<chrono::NaiveDateTime>,
    all_day: bool,
    #[getset(skip)]
    description: String,
    background_color: String,
    department: String,
    faculty: Option<String>,
    event_category: String,
    sites: Option<Vec<String>>,
    modules: Option<Vec<String>>,
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
        static LINEBREAKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\r\n|<br />)+").unwrap());
        html_escape::decode_html_entities(LINEBREAKS_REGEX.replace_all(&self.description, "\n").trim())
            .to_string()
    }
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

pub type GetLimitsResponse = (NaiveDate, NaiveDate);

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

    let (r, [y1, m1, d1, y2, m2, d2]) = LIMITS_REGEX
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
    Ok((date1, date2))
}
