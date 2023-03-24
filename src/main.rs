use std::{error::Error, process};
use std::fs::File;
use std::io::{BufReader, Read};

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand};
use fuzzy_finder::{item::Item, FuzzyFinder};

use tidescli::{Station, TidePredictions};

fn main() -> Result<(), Box<dyn Error>> {
    let station_bytes = &include_bytes!("../reference/stations.json")[..];

    let args = Cli::parse().command;
    match args {
        Commands::Stations(StationsArgs { fetch }) => {
            if fetch {
                todo!();
                // let station_bytes = // fetch stations from the net;
            }
            display_stations(station_bytes);
        },
        Commands::Tides(TidesArgs { station, date }) => {
            // fetch tides data for station
            let (_, _) = (station, date);
            todo!();
        },
    }

    Ok(())
}

fn display_stations(rdr: impl Read) {
    let stations = match tidescli::stations_from_reader(rdr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse stations data: {e}");
            process::exit(-1);
        },
    };
    let finder_items: Vec<Item<Station>> = stations
        .into_iter()
        .map(|s| Item::new(s.name.clone(), s))
        .collect();

    if let Ok(Some(station)) = FuzzyFinder::find(finder_items, 12) {
        println!("\r{0}\t{1}", station.id, station.name);
    }
}

fn _read_tides_reference_file() -> Result<TidePredictions, Box<dyn Error>> {
    let tides = File::open("./reference/tides.json")?;
    let tides = BufReader::new(tides);
    let tides = tidescli::tides_from_reader(tides)?;
    Ok(tides)
}

fn _read_stations_reference_file() -> Result<Vec<Station>, Box<dyn Error>> {
    let stations = File::open("./reference/stations.json")?;
    let stations = BufReader::new(stations);
    let stations = tidescli::stations_from_reader(stations)?;
    Ok(stations)
}

/// CLI hello!
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    Stations(StationsArgs),
    Tides(TidesArgs),
}

/// List all UK tidal stations supported by the UKHO.
#[derive(Args, Clone, Debug)]
struct StationsArgs {
    /// Fetch the current list of tidal stations from the UKHO web service.
    #[arg(short, long)]
    fetch: bool,
}

/// Display tide information for one station on a particular day.
#[derive(Args, Clone, Debug)]
struct TidesArgs {
    /// ID of the desired tidal station.
    #[arg(short, long)]
    station: String,
    /// Date of tidal data to display (YYYY-MM-DD).
    #[arg(short, long)]
    date: NaiveDate,
}
