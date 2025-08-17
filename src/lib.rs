//! rjw-uktides is a small library to help fetch and parse tide predictions data from the UK
//! Hydrographic Office [EasyTide] service.
//!
//! Tide predictions can be obtained for about 700 locations around Great Britain, Ireland, the
//! Channel Islands, and the Isle of Man. The data includes the predicted times of high and low
//! tides, so it's perfect for planning your next trip to the beach (please don't use it for
//! navigation).
//!
//! EasyTide is a publicly available web application that uses two unauthenticated JSON endpoints
//! to look up tide stations and tide predictions for those stations. No API key is needed.
//!
//! [EasyTide]: https://easytide.admiralty.co.uk/
//!
//! This library does not perform network IO itself, instead there are two pairs of functions that construct
//! the appropriate URLs for you to query with your preferred HTTP client, and parse the returned
//! JSON data:
//!
//! - [`stations_list_url()`] returns the URL that gives a list of tidal stations, which can then
//!   be parsed with [`stations_from_reader()`].
//! - [`tide_predictions_url()`] constructs a station-specific URL that gives tide prediction data,
//!   which can then be parsed with [`tides_from_reader()`].
//!
//! # Example usage
//!
//! Here's a full usage example using [`ureq`] to perform the GET request.
//!
//! [`ureq`]: https://docs.rs/ureq/latest/ureq/
//!
//! ```
//! # fn main() -> anyhow::Result<()> {
//! # use rjw_uktides::{Station, TidePredictions};
//! // Fetch the list of all stations.
//! let url = rjw_uktides::stations_list_url();
//! let body = ureq::get(url.as_str()).call()?.into_body().into_reader();
//! let stations: Vec<Station> = rjw_uktides::stations_from_reader(body)?;
//!
//! // Fetch tide predictions for the first station
//! let url = rjw_uktides::tide_predictions_url(&stations[0].id);
//! let body = ureq::get(url.as_str()).call()?.into_body().into_reader();
//! let predictions: TidePredictions = rjw_uktides::tides_from_reader(body)?;
//!
//! // Print the times of the high and low tides
//! for event in predictions.tidal_event_list {
//!     println!("{}    {}", event.date_time, event.event_type);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Typically you'll already know the station you want predictions for. Look up its ID, either from
//! the list of stations or from the value of the `PortID` query parameter on the EasyTide website.
//! For instance, [Sandown] on the Isle of Wight has an ID of "0053" (note that these are not numeric
//! IDs). We can look it up directly by creating a [`StationId`] from that string:
//!
//! [Sandown]: https://easytide.admiralty.co.uk/?PortID=0053
//!
//! ```
//! # fn main() -> anyhow::Result<()> {
//! # use rjw_uktides::{StationId, tide_predictions_url, tides_from_reader};
//! let sandown: StationId = "0053".into();
//! let url = tide_predictions_url(&sandown);
//! let body = ureq::get(url.as_str()).call()?.into_body().into_reader();
//! let sandown_predictions = tides_from_reader(body)?;
//! # Ok(())
//! # }
//!
//! ```
//!
//! # Main types
//!
//! The main structs of interest are:
//!
//! - [`TidePredictions`], which includes all of the prediction data, notably the times of high
//!   and low tides over the next few days;
//! - [`StationId`], which you need to use to obtain those predictions; and
//! - [`Station`], which contains more details about a particular tidal station.
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
