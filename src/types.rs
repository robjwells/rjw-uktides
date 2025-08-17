use serde::Deserialize;

/// Geographic coordinate represented as decimal degrees.
///
/// The contained `f64` is the decimal representation, its `String` representation
/// ([`Display`](std::fmt::Display)) is in sexagesimal (base-60) degrees, minutes and seconds
/// according to Annex D of [ISO 6709](https://en.wikipedia.org/wiki/ISO_6709).
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

/// Latitude and longitude of a tidal station.
///
/// It is not clear which coordinate system these are from, even the UKHO API documentation lists
/// it as "unspecified". Do not rely on the precision of the coordinates beyond specifying a
/// general location.
#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Coordinates {
    // NOTE that the order of the fields is important as this struct is represented by an array in
    // the JSON, longitude first.
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

/// Unique identifier for a tidal station used to look up tide predictions.
///
/// While most station IDs appear to be numeric (eg 0053 for Sandown), they are not as leading
/// zeroes are significant and some stations have a letter suffix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct StationId(pub String);

impl From<String> for StationId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for StationId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl std::fmt::Display for StationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Country in which a tidal station is located.
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

/// Details of a specific tidal station.
#[derive(Debug, Clone)]
pub struct Station {
    /// ID used to identify the station when requesting tidal predictions.
    pub id: StationId,
    /// The name of the location of the station.
    pub name: String,
    /// The country in which the station is located.
    pub country: Country,
    /// Geographic coordinates (latitude and longitude) of the station.
    pub location: Coordinates,
    /// Whether the station can provide continuous height predictions.
    ///
    /// These predictions are in metres every 30 minutes.
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

/// Tide prediction and related data for a particular station.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidePredictions {
    /// A note appended to the whole response that is typically safety-related.
    pub footer_note: String,
    /// Moon phase data.
    pub lunar_phase_list: Vec<LunarPhase>,
    /// Low- and high-tide event data.
    ///
    /// Typically these alternate between low and high tides, but note that in some
    /// locations [double tides] can occur.
    ///
    /// [double tides]: https://easytide.admiralty.co.uk/FAQs#:~:text=Double%20High%20Water
    pub tidal_event_list: Vec<TidalEvent>,
    /// Half-hourly tide height predictions.
    ///
    /// Note that not all stations provide these "continuous" heights, in which case this will be
    /// empty.
    pub tidal_height_occurrence_list: Vec<TidalHeightOccurence>,
}

// Custom Debug implementation to prevent the half-hourly tidal height predictions
// being included, which make the debug output *very* long.
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
    /// The predicted datetime at which the tide will occur, in the Europe/London timezone.
    // TODO: UKHO says "this may be missing if it is invalid". When/where?
    #[serde(deserialize_with = "crate::parse::datetime_without_tz")]
    pub date_time: jiff::Zoned,

    /// Discriminator between high and low tide.
    pub event_type: TidalEventType,

    /// Predicted tide height in metres.
    // TODO: UKHO says "this may be missing if it is invalid". When/where?
    pub height: Metres,

    /// Typically `null` in the (semi-)public API response.
    pub is_approximate_height: Option<String>,

    /// Typically `null` in the (semi-)public API response.
    pub is_approximate_time: Option<String>,
}

impl TidalEvent {
    /// The date on which the tide will occur, in the Europe/London timezone.
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

/// Predicted tide height in metres.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Metres(pub f64);

/// Represents either low or high tide.
///
/// The u8 discriminants match the numbers used in the semi-public API.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum TidalEventType {
    HighWater = 0,
    LowWater = 1,
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

/// Half-hourly prediction of tide height.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TidalHeightOccurence {
    /// Time of predicted height, in the Europe/London timezone.
    #[serde(deserialize_with = "crate::parse::zulu_datetime_to_zoned")]
    pub date_time: jiff::Zoned,
    /// Predicted tide height in metres.
    pub height: Metres,
}

/// Prediction of a particular lunar phase.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LunarPhase {
    /// Datetime of the lunar phase occurrence, in the Europe/London timezone.
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
