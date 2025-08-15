use std::str::FromStr;

use serde::{self, Deserialize, Deserializer, de::Unexpected};

use crate::types::{Coordinates, LunarPhaseType, Station, StationId, TidalEventType};

/// Parse ISO 8601 datetimes missing a timezone and with optional fractional seconds.
///
/// The Admiralty tides API returns dates as datetimes without a timezone specifier, and returns
/// some datetimes with a half-second appended (`.5`) and also without a datetime.
///
/// The API documentation on the Admirality website describes these dates and datetimes as being in
/// GMT, so they are parsed initially as "naive" datetimes, then given the UTC timezone, then
/// finally converted from UTC to the Europe/London timezone
///
/// # Errors
///
/// This function will return an error if `serde_json` fails to deserialize the data as a string
/// or if `jiff` fails to parse that string in `%Y-%m-%dT%H:%M:%S` format.
pub(crate) fn deserialize_datetime_without_tz<'de, D>(
    deserializer: D,
) -> Result<jiff::Zoned, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
    let value = String::deserialize(deserializer)?;
    // Split off any fractional seconds (".5").
    let date = value.split_once('.').map(|(d, _)| d).unwrap_or(&value);

    // The datetime in the JSON is in "GMT" (UTC), so the civil datetime (without a TZ)
    // first needs converting to UTC, then converting again to Europe/London to ensure
    // summer time gets applied.
    jiff::civil::DateTime::strptime(FORMAT, date)
        .and_then(|d| d.to_zoned(jiff::tz::TimeZone::UTC))
        .and_then(|d| d.in_tz("Europe/London"))
        .map_err(serde::de::Error::custom)
}

/// Parse an ISO 8601 datetime with a trailing Z.
///
/// The tidal height occurrences (the continuous height predictions) use this format.
pub(crate) fn deserialize_zulu_datetime_to_zoned<'de, D>(
    deserializer: D,
) -> Result<jiff::Zoned, D::Error>
where
    D: Deserializer<'de>,
{
    let date = String::deserialize(deserializer)?;
    jiff::Timestamp::from_str(&date)
        .and_then(|ts| ts.in_tz("Europe/London"))
        .map_err(serde::de::Error::custom)
}

/// Deserialize the "features" object of the GetStations endpoint result as `Station` structs.
///
/// The Admiralty public stations API contains unnecessary keys and unnecessarily nested data
/// (it appears to be from a GIS system). This parses the "features" object instead into a
/// `Vec` of [`Station`] structs which are simpler.
///
/// # Errors
///
/// This function will return an error if `serde_json` fails to parse the JSON into the format
/// expected from the Admiralty API. The conversion from the (internal) `StationFeature` structs
/// into `Station` structs will not fail.
fn deserialize_stations<'de, D>(deserializer: D) -> Result<Vec<Station>, D::Error>
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

/// Description of the Admirality stations API response wrapper.
///
/// The `type` field of the JSON response is always "FeatureCollection",
/// as it appears to come directly from a GIS system.
///
/// This is a level of nesting that is not necessary for users of this
/// crate, and so is just an intermediate representation from which
/// the (custom-deserialized) `Station` structs are pulled.
#[derive(Debug, Deserialize)]
pub(crate) struct StationsData {
    // Always 'FeatureCollection'
    #[serde(skip, rename = "type")]
    _type: String,

    #[serde(deserialize_with = "crate::parse::deserialize_stations")]
    pub(crate) features: Vec<Station>,
}

/// Station information as returned by the Admiralty API.
///
/// Not publicly exposed due to the redundant "type" key and the extra nesting.
#[derive(Debug, Deserialize)]
struct StationFeature {
    #[serde(rename = "type")]
    _type: String,
    /// Wrapper around the station's geographic coordinates.
    geometry: StationFeatureGeometry,
    /// Wrapper around the other important details about the station.
    properties: StationFeatureProperties,
}

/// Wrapper around a station's geographic coordinates.
#[derive(Debug, Deserialize)]
struct StationFeatureGeometry {
    #[serde(rename = "type")]
    _type: String, // "Point"
    coordinates: Coordinates,
}

/// Wrapper around the details of the station.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StationFeatureProperties {
    id: StationId,
    name: String,
    country: String,
    continuous_heights_available: bool,
}

impl<'de> Deserialize<'de> for TidalEventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = u64::deserialize(deserializer)?;
        match num {
            0 => Ok(Self::HighWater),
            1 => Ok(Self::LowWater),
            _ => {
                let unexp = Unexpected::Unsigned(num);
                let exp = &"an integer either 0 or 1";
                Err(serde::de::Error::invalid_value(unexp, exp))
            }
        }
    }
}

impl<'de> Deserialize<'de> for LunarPhaseType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = u64::deserialize(deserializer)?;
        match num {
            1 => Ok(Self::NewMoon),
            2 => Ok(Self::FirstQuarter),
            3 => Ok(Self::FullMoon),
            4 => Ok(Self::LastQuarter),
            _ => {
                let unexp = Unexpected::Unsigned(num);
                let exp = &"an integer from 1 to 4, inclusive.";
                Err(serde::de::Error::invalid_value(unexp, exp))
            }
        }
    }
}
