use serde::Deserialize;

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct DecimalDegrees(pub f64);

impl std::fmt::Display for DecimalDegrees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let abs_degrees = self.0.abs();
        let floor_abs_degrees = abs_degrees.floor();
        let abs_fractional = abs_degrees - floor_abs_degrees;

        let degrees = floor_abs_degrees;
        let minutes = (60.0 * abs_fractional).floor();
        let seconds = 3600.0 * abs_fractional - 60.0 * minutes;

        write!(
            f,
            "{d}°{m:02}′{s:02}″",
            d = degrees as u8,
            m = minutes as u8,
            s = seconds as u8
        )
    }
}

/// Geographic coordinates (latitude and longitude) of the station.
///
/// It is not clear which coordinate system these are from.
///
/// **Note** that the order of the fields is important as this struct
/// is represented by an array in the JSON, longitude first.
#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Coordinates {
    /// Longitude, in decimal degrees.
    pub longitude: DecimalDegrees,
    /// Latitude, in decimal degrees.
    pub latitude: DecimalDegrees,
}

impl std::fmt::Display for Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lon_char = if self.longitude.0 >= 0.0 { 'E' } else { 'W' };
        write!(
            f,
            "{lat}N {lon}{lc}",
            lat = self.latitude,
            lon = self.longitude,
            lc = lon_char,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct StationId(pub String);

impl From<String> for StationId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl std::str::FromStr for StationId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

impl std::fmt::Display for StationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Country {
    ChannelIslands,
    England,
    Ireland,
    IsleOfMan,
    NorthernIreland,
    Scotland,
    Wales,
}

impl std::str::FromStr for Country {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Channel Islands" => Ok(Self::ChannelIslands),
            "England" => Ok(Self::England),
            "Ireland" => Ok(Self::Ireland),
            "Isle of Man" => Ok(Self::IsleOfMan),
            "Northern Ireland" => Ok(Self::NorthernIreland),
            "Scotland" => Ok(Self::Scotland),
            "Wales" => Ok(Self::Wales),
            other => Err(format!("Unexpected country name {:?}", other)),
        }
    }
}

impl std::fmt::Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::ChannelIslands => "Channel Islands",
            Self::England => "England",
            Self::Ireland => "Ireland",
            Self::IsleOfMan => "Isle of Man",
            Self::NorthernIreland => "Northern Ireland",
            Self::Scotland => "Scotland",
            Self::Wales => "Wales",
        };
        write!(f, "{}", name)
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
    /// The country in which the station is placed.
    pub country: Country,
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

#[derive(Debug)]
pub enum StationDataSource {
    Cached,
    FetchLatest,
}

/// A wrapper for all of the tide prediction data from the UKHO API.
#[derive(Deserialize)]
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

// Custom Debug implementation to prevent the half-hourly tidal height predictions
// being included.
impl std::fmt::Debug for TidePredictions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TidePredictions")
            .field("footer_note", &self.footer_note)
            .field("lunar_phase_list", &self.lunar_phase_list)
            .field("tidal_event_list", &self.tidal_event_list)
            .field(
                "tidal_height_occurrence_list",
                &format!("[ {} heights ]", self.tidal_height_occurrence_list.len()),
            )
            .finish()
    }
}

/// An instance of low or high tide.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalEvent {
    /// The predicted datetime at which the tide measurement will occur.
    #[serde(deserialize_with = "crate::parse::datetime_without_tz")]
    pub date_time: jiff::Zoned,

    /// Discriminator between high and low tide.
    pub event_type: TidalEventType,

    /// Predicted tide height as a newtype-wrapped `f64`.
    pub height: Metres,

    /// Typically `null` in the (semi-)public API response.
    pub is_approximate_height: Option<String>,

    /// Typically `null` in the (semi-)public API response.
    pub is_approximate_time: Option<String>,
}

impl TidalEvent {
    pub fn date(&self) -> jiff::civil::Date {
        self.date_time.date()
    }
}

impl PartialEq for TidalEvent {
    fn eq(&self, other: &Self) -> bool {
        self.date_time == other.date_time
    }
}

impl Eq for TidalEvent {}

impl Ord for TidalEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date_time.cmp(&other.date_time)
    }
}

impl PartialOrd for TidalEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Tide height in metres as an `f64`, wrapped in a newtype to make the measurement unit clear.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Metres(pub f64);

/// Represents either low or high tide.
///
/// The UKHO API response encodes low tide as 1 and high tide as 0.
#[derive(Debug, Copy, Clone)]
pub enum TidalEventType {
    HighWater,
    LowWater,
}

impl std::fmt::Display for TidalEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            TidalEventType::HighWater => "High tide",
            TidalEventType::LowWater => "Low tide",
        };
        write!(f, "{text}")
    }
}

/// Prediction of the tide height in metres at a particular time.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalHeightOccurence {
    /// Time of prediction, typically every half-hour.
    #[serde(deserialize_with = "crate::parse::zulu_datetime_to_zoned")]
    pub date_time: jiff::Zoned,
    /// Predicted tide height as a newtype-wrapped `f64`.
    pub height: Metres,
}

/// Prediction of a particular lunar phase.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LunarPhase {
    /// Datetime of the lunar phase occurrence.
    #[serde(deserialize_with = "crate::parse::datetime_without_tz")]
    pub date_time: jiff::Zoned,

    /// The lunar phase itself.
    pub lunar_phase_type: LunarPhaseType,
}

/// Represents a particular phase of the moon.
///
/// The u8 discriminants match the numbers used in the semi-public API.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum LunarPhaseType {
    NewMoon = 1,
    FirstQuarter = 2,
    FullMoon = 3,
    LastQuarter = 4,
}
