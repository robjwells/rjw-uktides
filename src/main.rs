use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use tidescli::{Station, TidePredictions};

fn main() -> Result<(), Box<dyn Error>> {
    let tides = read_tides_reference_file()?;
    println!("{:#?}", tides);

    let stations = read_stations_reference_file()?;
    println!("{:#?}", stations);

    for s in stations {
        println!("{}\t{}", s.id, s.name);
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
