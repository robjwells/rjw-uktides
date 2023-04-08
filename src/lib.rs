mod parse;

use std::error::Error;

pub use parse::*;
use reqwest::blocking::Client;

const STATIONS_URL: &str = "https://easytide.admiralty.co.uk/Home/GetStations";

pub fn fetch_stations() -> Result<Vec<Station>, Box<dyn Error>> {
    let bytes = reqwest::blocking::get(STATIONS_URL)?.bytes()?;
    stations_from_reader(bytes.as_ref())
}

pub fn fetch_tides(station: StationId) -> Result<TidePredictions, Box<dyn Error>> {
    let url = "https://easytide.admiralty.co.uk/Home/GetPredictionData";
    let response = Client::new()
        .get(url)
        .query(&[("stationId", station.0)])
        .send()?;
    let body = response.text()?;
    tides_from_reader(body.as_bytes())
}
