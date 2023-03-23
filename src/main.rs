use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use fuzzy_finder::{item::Item, FuzzyFinder};

use tidescli::{Station, TidePredictions};

fn main() -> Result<(), Box<dyn Error>> {
    let _tides = read_tides_reference_file()?;
    // println!("{:#?}", tides);

    let mut stations = read_stations_reference_file()?;
    stations.sort_unstable();
    // println!("{:#?}", stations);

    let finder_items: Vec<Item<Station>> = stations
        .into_iter()
        .map(|s| Item::new(s.name.clone(), s))
        .collect();

    let selected = FuzzyFinder::find(finder_items, 12)?;
    if let Some(station) = selected {
        println!("\rGot this station from the fuzzy finder:");
        println!("{:?}", station);
    }

    Ok(())
}

fn read_tides_reference_file() -> Result<TidePredictions, Box<dyn Error>> {
    let tides = File::open("./reference/tides.json")?;
    let tides = BufReader::new(tides);
    let tides = tidescli::tides_from_reader(tides)?;
    Ok(tides)
}

fn read_stations_reference_file() -> Result<Vec<Station>, Box<dyn Error>> {
    let stations = File::open("./reference/stations.json")?;
    let stations = BufReader::new(stations);
    let stations = tidescli::stations_from_reader(stations)?;
    Ok(stations)
}
