mod error;
mod fetch;

use std::error::Error;

use clap::{Args, Parser, Subcommand};

pub use rjw_uktides::{Station, StationId};

use crate::fetch::{fetch_station_details, fetch_stations, fetch_stations_json, fetch_tides};

fn main() -> Result<(), Box<dyn Error>> {
    match Cli::parse() {
        Cli {
            subcommand: Some(Commands::List { json: true }),
            tides_args: None,
        } => {
            let mut json_reader = fetch_stations_json()?;
            std::io::copy(&mut json_reader, &mut std::io::stdout())?;
            // Ensure final newline to not mess-up terminals.
            println!();
        }
        Cli {
            subcommand: Some(Commands::List { json: false }),
            tides_args: None,
        } => {
            display_stations(fetch_stations()?);
        }
        Cli {
            subcommand: Some(Commands::Details { station_id }),
            tides_args: None,
        } => {
            println!("{:#?}", fetch_station_details(station_id)?);
        }
        Cli {
            subcommand: None,
            tides_args: Some(TidesArgs { station_id, format }),
        } => {
            let tides = fetch_tides(&station_id)?;
            for tide in tides.tidal_event_list {
                println!(
                    "{}    {}",
                    tide.date_time.strftime(&format),
                    tide.event_type
                );
            }
        }
        args @ Cli { .. } => unreachable!("{args:#?}"),
    }
    Ok(())
}

fn display_stations(mut s: Vec<Station>) {
    s.sort();
    for Station {
        id,
        name,
        country,
        location,
        continuous_heights_available,
    } in s
    {
        print!(
            "{:5}\t{:<34}\t{:16}\t{}",
            id,
            name,
            country.to_string(),
            location
        );
        if !continuous_heights_available {
            print!("\tno continuous heights")
        }
        println!()
    }
}

/// Fetch high and low tide times from the UK Hydrographic Office.
///
/// Data shown is that currently available from the web service used by
/// the official EasyTide website.
#[derive(Parser, Debug)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    // This extra wrapper seems necessary to get the arg/command conflict behaviour.
    #[command(flatten)]
    tides_args: Option<TidesArgs>,

    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// List the details of all available tide stations.
    ///
    /// By default this displays stations in a text format.
    List {
        /// Print the JSON data received from EasyTide.
        #[arg(short, long, default_value = "false")]
        json: bool,
    },
    /// Show the details of one station.
    Details {
        /// ID of the desired tidal station.
        station_id: StationId,
    },
}

/// Display tide information for one station on a particular day.
#[derive(Args, Clone, Debug)]
struct TidesArgs {
    /// ID of the desired tidal station.
    #[arg(short, long, value_name = "STATION_ID")]
    station_id: StationId,
    /// strftime format string to use for tidal event datetimes
    #[arg(short, long, default_value = "%Y-%m-%d %H:%M %Z")]
    format: String,
}
