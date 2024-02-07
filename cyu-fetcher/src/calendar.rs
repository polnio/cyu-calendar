use crate::errors::Error;
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::error::Error as _;

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
        /* let (day, time) = s.split_once('T').ok_or(Error::custom("Invalid date"))?;
        let (year, month, day) = day
            .split('-')
            .map(|s| s.parse::<u32>().unwrap())
            .collect_tuple()
            .ok_or(Error::custom("Invalid date"))?;
        let (hour, minute, second) = time
            .split(':')
            .map(|s| s.parse::<u32>().unwrap())
            .collect_tuple()
            .ok_or(Error::custom("Invalid date"))?;
        let date = NaiveDate::from_ymd_opt(year as i32, month, day)
            .ok_or(Error::custom("Invalid date"))?
            .and_hms_opt(hour, minute, second)
            .ok_or(Error::custom("Invalid date"))?; */

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
        /* let date = s.map(|s| {
            Some(chrono::NaiveDateTime::parse_from_str(
                &s,
                "%Y-%m-%dT%H:%M:%S",
            ))
        }); */
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
    description: String,
    background_color: String,
    department: String,
    faculty: Option<String>,
    event_category: String,
    sites: Option<Vec<String>>,
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

    println!("{}", serde_json::to_string(&remote_payload).unwrap());

    let response = requester
        .post("https://services-web.cyu.fr/calendar/Home/GetCalendarData")
        .form(&remote_payload)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Cookie", query.token)
        .send()
        .await
        .map_err(|_| Error::Remote)?;

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

    // println!("{:#?}", calendar);

    Ok(calendar)
}
