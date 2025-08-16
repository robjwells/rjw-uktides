mod error;
mod parse;
mod types;

use std::io::Read;

use url::Url;

pub use crate::error::Error;
pub use crate::types::{Station, StationDataSource, StationId, TidePredictions};

const STATIONS_JSON_CACHED: &[u8] = include_bytes!("../stations.json");

const STATIONS_URL: &str = "https://easytide.admiralty.co.uk/Home/GetStations";
const TIDES_URL: &str = "https://easytide.admiralty.co.uk/Home/GetPredictionData";

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
/// The data should be JSON sourced from the UKHO (semi-)public
/// Home/GetPredictions endpoint.
pub fn tides_from_reader(rdr: impl Read) -> Result<TidePredictions, Error> {
    serde_json::from_reader(rdr).map_err(Error::Parse)
}

/// Attempt to extract tide station information from the reader.
///
/// The data should be JSON sourced from the UKHO (semi-)public
/// Home/GetStations endpoint. The "features" property of the returned
/// JSON is returned as a `Vec` of `Station`.
///
/// The [`Station`] struct simplifies the nested structure of the
/// JSON returned by the GetStations endpoint.
pub fn stations_from_reader(rdr: impl Read) -> Result<Vec<Station>, Error> {
    serde_json::from_reader(rdr)
        .map(|sd: crate::parse::StationsData| sd.features)
        .map_err(Error::Parse)
}

/// Get the station information valid at the time rjw_uktides was released.
///
/// This is helpful to look up station information without making a network call,
/// but may become out of date as the UKHO changes the available stations.
///
/// Note that this function parses the JSON embedded in the library, so it is not free.
pub fn cached_stations() -> Vec<Station> {
    stations_from_reader(STATIONS_JSON_CACHED)
        .expect("Embedded stations data must be verified as valid.")
}
