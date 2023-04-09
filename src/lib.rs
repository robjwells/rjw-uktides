mod parse;

use std::error::Error;

pub use parse::*;
use reqwest::blocking::Client;

const STATIONS_BAKED_BYTES: &[u8] = include_bytes!("../stations.json");

const STATIONS_URL: &str = "https://easytide.admiralty.co.uk/Home/GetStations";

pub fn cached_stations() -> Vec<Station> {
    stations_from_reader(STATIONS_BAKED_BYTES)
        .expect("Embedded stations data must be verified as valid.")
}

pub fn fetch_stations() -> Result<Vec<Station>, Box<dyn Error>> {
    let bytes = reqwest::blocking::get(STATIONS_URL)?.bytes()?;
    stations_from_reader(bytes.as_ref())
}

pub fn fetch_tides(station: &StationId) -> Result<TidePredictions, Box<dyn Error>> {
    let url = "https://easytide.admiralty.co.uk/Home/GetPredictionData";
    let response = Client::new()
        .get(url)
        .query(&[("stationId", station.0.as_str())])
        .send()?;
    let body = response.text()?;
    tides_from_reader(body.as_bytes())
}

#[derive(Debug)]
pub enum StationDataSource {
    Cached,
    FetchLatest,
}

pub fn station_details(
    id: &StationId,
    source: StationDataSource,
) -> Result<Station, Box<dyn Error>> {
    use self::StationDataSource::*;
    let stations = match source {
        Cached => cached_stations(),
        FetchLatest => fetch_stations()?,
    };
    stations
        .into_iter()
        .find(|s| &s.id == id)
        .ok_or_else(|| format!("No station with ID {}", id).into())
}
