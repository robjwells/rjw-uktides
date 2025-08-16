mod parse;
mod types;

use std::io::Read;

use url::Url;

pub use crate::types::{Station, StationDataSource, StationId, TidePredictions};

const STATIONS_JSON_CACHED: &[u8] = include_bytes!("../stations.json");

const STATIONS_URL: &str = "https://easytide.admiralty.co.uk/Home/GetStations";
const TIDES_URL: &str = "https://easytide.admiralty.co.uk/Home/GetPredictionData";

#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: This is a dummy implementation
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

pub fn cached_stations() -> Vec<Station> {
    stations_from_reader(STATIONS_JSON_CACHED)
        .expect("Embedded stations data must be verified as valid.")
}

/// Get the URL for information on all available stations.
pub fn stations_list_url() -> Url {
    STATIONS_URL
        .parse()
        .expect("Station list URL is known to be valid")
}

/// Construct a tide-prediction URL for the given station.
pub fn tide_predictions_url(station: &StationId) -> Url {
    Url::parse_with_params(TIDES_URL, &[("stationID", &station.0)])
        .expect("Tide predictions URL is known to be valid")
}

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
pub fn tides_from_reader(rdr: impl Read) -> Result<TidePredictions, Error> {
    serde_json::from_reader(rdr).map_err(Error::Parse)
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
/// let stations = rjw_uktides::stations_from_reader(bufreader)
///     .expect("Failed to read file as stations data.");
/// ```
pub fn stations_from_reader(rdr: impl Read) -> Result<Vec<Station>, Error> {
    serde_json::from_reader(rdr)
        .map(|sd: crate::parse::StationsData| sd.features)
        .map_err(Error::Parse)
}
