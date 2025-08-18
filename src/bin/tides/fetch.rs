use crate::error::TidesError;

use rjw_uktides::{
    Station, StationId, TidePredictions, stations_from_reader, stations_list_url,
    tide_predictions_url, tides_from_reader,
};

pub(crate) fn fetch_stations_json() -> Result<ureq::BodyReader<'static>, TidesError> {
    ureq::get(stations_list_url().as_str())
        .call()
        .map(|r| r.into_body().into_reader())
        .map_err(TidesError::Fetch)
}

pub fn fetch_stations() -> Result<Vec<Station>, TidesError> {
    let reader = fetch_stations_json()?;
    stations_from_reader(reader).map_err(TidesError::Library)
}

pub fn fetch_tides(station: &StationId) -> Result<TidePredictions, TidesError> {
    let reader = ureq::get(tide_predictions_url(station).as_str())
        .call()
        .map(|r| r.into_body().into_reader())
        .map_err(TidesError::Fetch)?;
    tides_from_reader(reader).map_err(TidesError::Library)
}

pub fn fetch_station_details(id: StationId) -> Result<Station, TidesError> {
    fetch_stations()?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| TidesError::NoSuchStation(id))
}
