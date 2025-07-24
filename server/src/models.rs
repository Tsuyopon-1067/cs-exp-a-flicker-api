use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoData {
    #[serde(
        deserialize_with = "deserialize_datetime",
        serialize_with = "serialize_datetime_as_string"
    )]
    pub date: DateTime<Utc>,
    pub lat: f64,
    pub lon: f64,
    pub url: String,
}

// 文字列をDateTime<Utc>に変換するdeserializer
fn deserialize_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let naive_dt =
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)?;
    Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc))
}

// DateTime<Utc>を "YYYY-MM-DD HH:MM:SS" 形式の文字列に変換するserializer
fn serialize_datetime_as_string<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = date.format("%Y-%m-%d %H:%M:%S").to_string();
    serializer.serialize_str(&s)
}

impl PhotoData {
    pub fn new(date_str: &str, lat: f64, lon: f64, url: &str) -> Result<Self, Box<dyn Error>> {
        let naive_dt = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S")?;
        let date = DateTime::from_naive_utc_and_offset(naive_dt, Utc);
        Ok(PhotoData {
            date,
            lat,
            lon,
            url: url.to_string(),
        })
    }
}
