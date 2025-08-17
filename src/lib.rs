mod error;
mod parse;
mod types;

use std::io::Read;

use url::Url;

pub use crate::error::Error;
pub use crate::types::{
    Coordinates, Country, DecimalDegrees, LunarPhase, LunarPhaseType, Metres, Station, StationId,
    TidalEvent, TidalEventType, TidalHeightOccurence, TidePredictions,
};

/// Stations list HTTP endpoint
const STATIONS_URL: &str = "https://easytide.admiralty.co.uk/Home/GetStations";
/// Station-specific tide predictions HTTP endpoint (requires `stationID` query parameter)
const TIDES_URL: &str = "https://easytide.admiralty.co.uk/Home/GetPredictionData";

/// Get the URL for information on all available stations.
pub fn stations_list_url() -> Url {
    STATIONS_URL
        .parse()
        .expect("Station list URL is known to be valid")
}

/// Parse a tide stations list from the reader.
///
/// `rdr` should provide JSON sourced from the URL returned by [`stations_list_url()`].
///
/// An error is returned if any relevant part of the JSON cannot be parsed as expected.
pub fn stations_from_reader(rdr: impl Read) -> Result<Vec<Station>, Error> {
    serde_json::from_reader(rdr)
        .map(|sd: crate::parse::StationsData| sd.features)
        .map_err(Error::Parse)
}

/// Construct a tide-prediction URL for the given station.
pub fn tide_predictions_url(station: &StationId) -> Url {
    Url::parse_with_params(TIDES_URL, &[("stationID", &station.0)])
        .expect("Tide predictions URL is known to be valid")
}

/// Parse tide predictions for a specific station from the reader.
///
/// `rdr` should provide JSON sourced from the URL returned by [`tide_predictions_url()`].
///
/// An error is returned if any relevant part of the JSON cannot be parsed as expected.
pub fn tides_from_reader(rdr: impl Read) -> Result<TidePredictions, Error> {
    serde_json::from_reader(rdr).map_err(Error::Parse)
}
