use chrono::{DateTime, NaiveDateTime, Utc};
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct PhotoData {
    #[serde(deserialize_with = "deserialize_datetime")]
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
