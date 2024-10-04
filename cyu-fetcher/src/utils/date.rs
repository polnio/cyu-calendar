use derive_more::derive::{Deref, From};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, From)]
pub struct CyuDate(chrono::NaiveDate);
#[derive(Debug, Default, Clone, PartialEq, Eq, Deref, From)]
pub struct CyuDateTime(chrono::NaiveDateTime);

impl CyuDate {
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        chrono::NaiveDate::from_ymd_opt(year, month, day).map(Self::from)
    }
    pub fn today() -> Self {
        chrono::Utc::now().date_naive().into()
    }
}

impl CyuDateTime {
    pub fn new(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<Self> {
        let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
        let time = chrono::NaiveTime::from_hms_opt(hour, min, sec)?;
        Some(chrono::NaiveDateTime::new(date, time).into())
    }
    pub fn today() -> Self {
        chrono::Utc::now().naive_local().into()
    }
}

impl Display for CyuDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format("%Y-%m-%d"))
    }
}

impl Display for CyuDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format("%Y-%m-%dT%H:%M:%S"))
    }
}

impl Serialize for CyuDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'a> Deserialize<'a> for CyuDate {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        let date = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|_| serde::de::Error::custom("Invalid date"))?;
        Ok(Self(date))
    }
}

impl Serialize for CyuDateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'a> Deserialize<'a> for CyuDateTime {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;

        let date = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
            .map_err(|_| serde::de::Error::custom("Invalid date"))?;
        Ok(Self(date))
    }
}
