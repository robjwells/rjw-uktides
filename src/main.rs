use std::fs::File;
use std::io::{BufReader, Read};
use std::{error::Error, process};

use chrono::NaiveDate;
use chrono_tz::Europe::London;
use clap::{Args, Parser, Subcommand};

use tidescli::{Station, StationId, TidePredictions};

fn main() -> Result<(), Box<dyn Error>> {
    let station_bytes = &include_bytes!("../reference/stations.json")[..];

    let args = Cli::parse().command;
    match args {
        Commands::ListStations(StationsArgs { fetch }) => {
            if fetch {
                todo!();
                // let station_bytes = // fetch stations from the net;
            }
            display_stations(station_bytes);
        }
        Commands::Tides(tides_args) => {
            // fetch tides data for station
            println!("{:#?}", tides_args);

            let s = tides_args.station;

            let url = format!(
                "https://easytide.admiralty.co.uk/Home/GetPredictionData?stationId={}",
                s
            );

            let response = reqwest::blocking::get(url)?;
            let body = response.text().unwrap();
            match tidescli::tides_from_reader(body.as_bytes()) {
                Ok(tides) => {
                    do_something_with_tides(tides, tides_args.date);
                }
                Err(e) => {
                    eprintln!("Got error: {e:?}\n\n");
                    eprintln!("Response body was:\n{}\n", body);
                }
            }
        }
    }
    Ok(())
}

fn do_something_with_tides(tides: TidePredictions, date: NaiveDate) {
    let todays_tides = tides
        .tidal_event_list
        .into_iter()
        .filter(|e| e.date == date);

    for tide in todays_tides {
        let time = tide
            .date_time
            .with_timezone(&London)
            .format("%l.%M%p")
            .to_string()
            .to_ascii_lowercase();
        println!("{}\t{:#?}", tide.event_type, time);
    }
}

fn display_stations(rdr: impl Read) {
    match tidescli::stations_from_reader(rdr) {
        Ok(mut s) => {
            s.sort();
            for Station { id, name, .. } in s {
                println!("{}\t{}", id, name);
            }
        }
        Err(e) => {
            eprintln!("Failed to parse stations data: {e}");
            process::exit(-1);
        }
    };
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
    ListStations(StationsArgs),
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
    station: StationId,
    /// Date of tidal data to display (YYYY-MM-DD).
    #[arg(short, long)]
    date: NaiveDate,
}
