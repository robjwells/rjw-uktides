#![allow(dead_code)]
use std::error::Error;

use chrono::{DateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer};
use serde_repr::Deserialize_repr;

pub fn predictions_from_json(input: &str) -> Result<PredictionData, Box<dyn Error>> {
    let tides: PredictionData = serde_json::from_str(input)?;
    Ok(tides)
}

pub fn stations_from_json(input: &str) -> Result<Vec<Station>, Box<dyn Error>> {
    let stations: StationsData = serde_json::from_str(input)?;
    Ok(stations.features)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictionData {
    pub footer_note: String,
    pub lunar_phase_list: Vec<LunarPhase>,
    pub tidal_event_list: Vec<TidalEvent>,
    pub tidal_height_occurrence_list: Vec<TidalHeightOccurence>,
}

fn deserialize_datetime_without_tz<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
    let value = String::deserialize(deserializer)?;
    let date = value.find('.').map_or(&value[..], |idx| &value[..idx]);
    Utc.datetime_from_str(date, FORMAT)
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalEvent {
    #[serde(deserialize_with = "deserialize_datetime_without_tz")]
    pub date: DateTime<Utc>,

    #[serde(deserialize_with = "deserialize_datetime_without_tz")]
    pub date_time: DateTime<Utc>,

    pub event_type: TidalEventType,
    pub height: Metres,
    pub is_approximate_height: Option<String>,
    pub is_approximate_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Metres(pub f64);

// var LowWater = 1;  // The value of Low Water from dbase is "1".
// var HighWater = 0; // The value of Low Water from dbase is "0".
#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub enum TidalEventType {
    HighWater = 0,
    LowWater = 1,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalHeightOccurence {
    pub date_time: DateTime<Utc>,
    pub height: Metres,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LunarPhase {
    pub date_time: String,
    pub lunar_phase_type: LunarPhaseType,
}

// var newMoon = 1; // The value of New Moon from dbase is "1".
// var firstQuarter = 2; // The value of First Quarter from dbase is "2".
// var fullMoon = 3; // The value of Low Full Moon dbase is "3".
// var lastQuarter = 4; // The value of Last Quarter from dbase is "4".
#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
pub enum LunarPhaseType {
    NewMoon = 1,
    FirstQuarter = 2,
    FullMoon = 3,
    LastQuarter = 4,
}

#[derive(Debug, Deserialize)]
pub struct StationsData {
    // Always 'FeatureCollection'
    #[serde(skip, rename = "type")]
    _type: String,

    #[serde(deserialize_with = "deser_stations")]
    pub features: Vec<Station>,
}

#[derive(Debug)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub country: String,
    pub location: Coordinates,
    pub continuous_heights_available: bool,
}

#[derive(Debug, Deserialize)]
struct StationFeature {
    #[serde(rename = "type")]
    _type: String,
    geometry: StationFeatureGeometry,
    properties: StationFeatureProperties,
}

fn deser_stations<'de, D>(deserializer: D) -> Result<Vec<Station>, D::Error>
where
    D: Deserializer<'de>,
{
    let features: Vec<StationFeature> = Vec::deserialize(deserializer)?;
    let stations = features
        .into_iter()
        .map(|feature| Station {
            id: feature.properties.id,
            name: feature.properties.name,
            country: feature.properties.country,
            location: feature.geometry.coordinates,
            continuous_heights_available: feature.properties.continuous_heights_available,
        })
        .collect();
    Ok(stations)
}

#[derive(Debug, Deserialize)]
struct StationFeatureGeometry {
    #[serde(rename = "type")]
    _type: String, // "Point"
    coordinates: Coordinates,
}

#[derive(Debug, Deserialize)]
pub struct Coordinates {
    // Order is important here as it's represented by an array in the JSON.
    pub lon: f64,
    pub lat: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StationFeatureProperties {
    id: String,
    name: String,
    country: String,
    continuous_heights_available: bool,
}
