use rjw_uktides::{
    Error, Station, StationDataSource, StationId, TidePredictions, stations_from_reader,
    stations_list_url, tide_predictions_url, tides_from_reader,
};

pub fn fetch_stations<'a>() -> Result<Vec<Station>, Error<'a>> {
    ureq::get(stations_list_url().as_str())
        .call()
        .map_err(Error::FetchError)
        .map(|r| r.into_body().into_reader())
        .and_then(stations_from_reader)
}

pub fn fetch_tides(station: &StationId) -> Result<TidePredictions, Error<'static>> {
    ureq::get(tide_predictions_url(station).as_str())
        .call()
        .map_err(Error::FetchError)
        .map(|r| r.into_body().into_reader())
        .and_then(tides_from_reader)
}

#[allow(dead_code)]
pub fn station_details<'a>(
    id: &'a StationId,
    source: StationDataSource,
) -> Result<Station, Error<'a>> {
    use StationDataSource::*;
    match source {
        Cached => rjw_uktides::cached_stations(),
        FetchLatest => fetch_stations()?,
    }
    .into_iter()
    .find(|s| &s.id == id)
    .ok_or_else(|| Error::NoSuchStation(id))
}
