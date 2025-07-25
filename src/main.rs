use std::error::Error;

use clap::{Args, Parser, Subcommand};

use rjw_uktides::{fetch_tides, Station, StationId};

fn main() -> Result<(), Box<dyn Error>> {
    let Cli {
        tides_args,
        subcommand,
    } = Cli::parse();
    match (tides_args, subcommand) {
        (None, Some(Commands::ListStations(StationsArgs { fetch }))) => {
            let stations = if fetch {
                rjw_uktides::fetch_stations()?
            } else {
                rjw_uktides::cached_stations()
            };
            display_stations(stations);
        }
        (Some(tides_args), None) => {
            let tides = fetch_tides(&tides_args.station);
            match tides {
                Ok(tides) => {
                    for tide in tides.tidal_event_list {
                        println!("{:?},{}", tide.date_time, tide.event_type);
                    }
                }
                Err(e) => {
                    eprintln!("Got error: {e:?}\n\n");
                }
            }
        }
        misc => {
            eprintln!("Unexpected argument state:\n{:#?}", misc);
            return Err("Unexpected argument state.".to_owned().into());
        }
    }
    Ok(())
}

fn display_stations(mut s: Vec<Station>) {
    s.sort();
    for Station { id, name, .. } in s {
        println!("{}\t{}", id, name);
    }
}

/// Fetch high and low tide times from the UK Hydrographic Office.
///
/// Data shown is that currently available from the web service used by
/// the official EasyTide website.
#[derive(Parser, Debug)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(flatten)]
    tides_args: Option<TidesArgs>,

    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    ListStations(StationsArgs),
}

/// List all UK tidal stations supported by the UKHO.
#[derive(Args, Clone, Debug)]
struct StationsArgs {
    /// Fetch the current list of tidal stations from the UKHO web service.
    ///
    /// If this argument is omitted, stations data built into the binary will be used.
    #[arg(short, long)]
    fetch: bool,
}

/// Display tide information for one station on a particular day.
#[derive(Args, Clone, Debug)]
struct TidesArgs {
    /// ID of the desired tidal station.
    #[arg(short, long)]
    station: StationId,
}
