use std::io::Read;
use std::str::FromStr;
use std::{error::Error, fmt::Display};

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use serde::{self, de::Unexpected, Deserialize, Deserializer};

/// Attempt to parse data from the reader as tide predictions.
///
/// The data should be JSON sourced from the Admiralty (semi-)public
/// Home/GetPredictions endpoint.
///
/// # Errors
///
/// This function will return an error if it cannot parse the data
/// from the reader as JSON or as JSON that encodes tide predictions.
/// Currently the error will only be a `serde_json::Error` but is
/// boxed to hide changes in the implementation in the future.
///
/// (`serde_json::Error` itself just contains a Boxed error, but this
/// extra indirection isn't expected to cause performance problems as
/// this function is effectively the "end of the line" for the error.)
///
/// # Examples
/// ```
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let file = File::open("./reference/tides.json")
///     .expect("Failed to open tides reference file.");
/// let bufreader = BufReader::new(file);
/// let tides = rjw_uktides::tides_from_reader(bufreader)
///     .expect("Failed to read file as tides data.");
/// ```
pub fn tides_from_reader(rdr: impl Read) -> Result<TidePredictions, Box<dyn Error>> {
    let tides = serde_json::from_reader(rdr)?;
    Ok(tides)
}

/// Attempt to extract tide station information from the reader.
///
/// The data should be JSON sourced from the Admiralty (semi-)public
/// Home/GetStations endpoint. The "features" property of the returned
/// JSON is returned as a `Vec` of `Station`.
///
/// The [`Station`] struct simplifies the nested structure of the
/// JSON returned by the GetStations endpoint.
///
/// # Errors
///
/// This function will return an error if it cannot parse the data
/// from the reader as JSON or as JSON that encodes station data.
/// Currently the error will only be a `serde_json::Error` but is
/// boxed to hide changes in the implementation in the future.
///
/// (`serde_json::Error` itself just contains a Boxed error, but this
/// extra indirection isn't expected to cause performance problems as
/// this function is effectively the "end of the line" for the error.)
///
/// # Examples
/// ```
/// use std::fs::File;
/// use std::io::BufReader;
///
/// let file = File::open("./reference/stations.json")
///     .expect("Failed to open stations reference file.");
/// let bufreader = BufReader::new(file);
/// let stations = tidescli::stations_from_reader(bufreader)
///     .expect("Failed to read file as stations data.");
/// ```
pub fn stations_from_reader(rdr: impl Read) -> Result<Vec<Station>, Box<dyn Error>> {
    let stations: StationsData = serde_json::from_reader(rdr)?;
    Ok(stations.features)
}

/// Parse ISO 8601 datetimes missing a timezone and with optional fractional seconds as UTC.
///
/// The Admiralty tides API returns dates as datetimes without a timezone specifier, and returns
/// some datetimes with a half-second appended (`.5`) and also without a datetime.
///
/// The API documentation on the Admirality website describes these dates and datetimes as being in
/// GMT, so they are parsed here as Chrono UTC datetimes.
///
/// # Errors
///
/// This function will return an error if `serde_json` fails to deserialize the data as a `String`
/// or if `chrono` fails to parse that `String` in `%Y-%m-%dT%H:%M:%S` format.
fn deserialize_datetime_without_tz<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
    let value = String::deserialize(deserializer)?;
    let date = value
        .rfind('.')
        .map_or_else(|| value.as_str(), |idx| value.split_at(idx).0);
    Utc.datetime_from_str(date, FORMAT)
        .map_err(serde::de::Error::custom)
}

fn deserialize_date_without_tz<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let datetime = deserialize_datetime_without_tz(deserializer);
    match datetime {
        Ok(d) => Ok(d.date_naive()),
        Err(e) => {
            eprintln!("Failed to parse naive date as naive datetime: {e:?}");
            Err(e)
        }
    }
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

/// A wrapper for all of the tide prediction data from the Admiralty API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidePredictions {
    /// A note appended to the whole response.
    ///
    /// This is usually a warning that the "high water duration period can occur over an extended
    /// time period."
    pub footer_note: String,
    /// Moon phase data.
    pub lunar_phase_list: Vec<LunarPhase>,
    /// Low- and high-tide event data.
    ///
    /// These include alternating low and high tides, their predicted height and when they will
    /// occur.
    pub tidal_event_list: Vec<TidalEvent>,
    /// Half-hourly tide height predictions.
    pub tidal_height_occurrence_list: Vec<TidalHeightOccurence>,
}

/// An instance of low or high tide.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalEvent {
    /// The day on which this tide occurs.
    #[serde(deserialize_with = "deserialize_date_without_tz")]
    pub date: NaiveDate,

    /// The predicted datetime at which the tide measurement will occur.
    #[serde(deserialize_with = "deserialize_datetime_without_tz")]
    pub date_time: DateTime<Utc>,

    /// Discriminator between high and low tide.
    pub event_type: TidalEventType,

    /// Predicted tide height as a newtype-wrapped `f64`.
    pub height: Metres,

    /// Typically `null` in the (semi-)public API response.
    pub is_approximate_height: Option<String>,

    /// Typically `null` in the (semi-)public API response.
    pub is_approximate_time: Option<String>,
}

/// Tide height in metres as an `f64`, wrapped in a newtype to make the measurement unit clear.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Metres(pub f64);

/// Represents either low or high tide.
///
/// The Admiralty API response encodes low tide as 1 and high tide as 0.
#[derive(Debug, Copy, Clone)]
pub enum TidalEventType {
    HighWater,
    LowWater,
}

impl Display for TidalEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            TidalEventType::HighWater => "High tide",
            TidalEventType::LowWater => "Low tide",
        };
        write!(f, "{text}")
    }
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

/// Prediction of the tide height in metres at a particular time.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalHeightOccurence {
    /// Time of prediction, typically every half-hour.
    pub date_time: DateTime<Utc>,
    /// Predicted tide height as a newtype-wrapped `f64`.
    pub height: Metres,
}

/// Prediction of a particular lunar phase.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LunarPhase {
    /// Datetime of the lunar phase occurrence.
    #[serde(deserialize_with = "deserialize_datetime_without_tz")]
    pub date_time: DateTime<Utc>,

    /// The lunar phase itself.
    pub lunar_phase_type: LunarPhaseType,
}

// var newMoon = 1; // The value of New Moon from dbase is "1".
// var firstQuarter = 2; // The value of First Quarter from dbase is "2".
// var fullMoon = 3; // The value of Low Full Moon dbase is "3".
// var lastQuarter = 4; // The value of Last Quarter from dbase is "4".

/// Represents a particular phase of the moon.
///
/// The Admiralty API encodes the lunar phase as an integer in this order:
///
/// 1. New moon.
/// 2. First quarter moon.
/// 3. Full moon.
/// 4. Last quarter moon.
#[derive(Debug, Copy, Clone)]
pub enum LunarPhaseType {
    NewMoon,
    FirstQuarter,
    FullMoon,
    LastQuarter,
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

/// Description of the Admirality stations API response wrapper.
///
/// The `type` field of the JSON response is always "FeatureCollection",
/// as it appears to come directly from a GIS system.
///
/// This is a level of nesting that is not necessary for users of this
/// crate, and so is just an intermediate representation from which
/// the (custom-deserialized) `Station` structs are pulled.
#[derive(Debug, Deserialize)]
struct StationsData {
    // Always 'FeatureCollection'
    #[serde(skip, rename = "type")]
    _type: String,

    #[serde(deserialize_with = "deserialize_stations")]
    features: Vec<Station>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StationId(pub String);

impl From<String> for StationId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl FromStr for StationId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl Display for StationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Details of a specific tidal measurement station.
#[derive(Debug, Clone)]
pub struct Station {
    /// ID used to identify the station when requesting tidal predictions.
    ///
    /// The ID appears numeric but leading zeroes are required when making the tidal prediction
    /// request, hence it is deserialized as a newtype-wrapped `String`.
    pub id: StationId,
    /// The name of the location of the station.
    pub name: String,
    /// The "country" in which the station is placed.
    ///
    /// The possibilities are:
    ///
    /// - "Channel Islands"
    /// - "England"
    /// - "Isle of Man"
    /// - "Northern Ireland"
    /// - "Scotland"
    /// - "Wales"
    pub country: String,
    /// Geographic coordinates (latitude and longitude) of the station.
    ///
    /// It is not clear which coordinate system these are from; perhaps WGS 84.
    pub location: Coordinates,
    /// Whether the station can provide continuous height measurements.
    pub continuous_heights_available: bool,
}

impl PartialEq for Station {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Station {}

impl Ord for Station {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Station {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct DecimalDegrees(f64);

/// Geographic coordinates (latitude and longitude) of the station.
///
/// It is not clear which coordinate system these are from; perhaps WGS 84.
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Coordinates {
    // Order is important here as this struct is represented by an array in the JSON.
    /// Longitude, in decimal degrees.
    pub longitude: DecimalDegrees,
    /// Latitude, in decimal degrees.
    pub latitude: DecimalDegrees,
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
