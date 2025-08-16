use crate::error::TidesError;

use rjw_uktides::{
    Station, StationDataSource, StationId, TidePredictions, stations_from_reader,
    stations_list_url, tide_predictions_url, tides_from_reader,
};

pub fn fetch_stations<'a>() -> Result<Vec<Station>, TidesError<'a>> {
    let rdr = ureq::get(stations_list_url().as_str())
        .call()
        .map(|r| r.into_body().into_reader())
        .map_err(TidesError::Fetch)?;
    stations_from_reader(rdr).map_err(TidesError::Library)
}

pub fn fetch_tides(station: &StationId) -> Result<TidePredictions, TidesError<'static>> {
    let reader = ureq::get(tide_predictions_url(station).as_str())
        .call()
        .map(|r| r.into_body().into_reader())
        .map_err(TidesError::Fetch)?;
    tides_from_reader(reader).map_err(TidesError::Library)
}

#[allow(dead_code)]
pub fn station_details(
    id: &StationId,
    source: StationDataSource,
) -> Result<Station, TidesError<'_>> {
    use StationDataSource::*;
    match source {
        Cached => rjw_uktides::cached_stations(),
        FetchLatest => fetch_stations()?,
    }
    .into_iter()
    .find(|s| &s.id == id)
    .ok_or_else(|| TidesError::NoSuchStation(id))
}
