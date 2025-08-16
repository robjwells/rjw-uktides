mod error;
mod fetch;

use std::error::Error;

use clap::{Args, Parser, Subcommand};

pub use rjw_uktides::{Station, StationId};

fn main() -> Result<(), Box<dyn Error>> {
    let Cli {
        tides_args,
        subcommand,
    } = Cli::parse();
    match (tides_args, subcommand) {
        (None, Some(Commands::ListStations)) => {
            display_stations(fetch::fetch_stations()?);
        }
        (Some(tides_args), None) => {
            let tides = fetch::fetch_tides(&tides_args.station);
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
    #[command(flatten)]
    tides_args: Option<TidesArgs>,

    #[command(subcommand)]
    subcommand: Option<Commands>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    ListStations,
    // TODO: Add subcommand to display single station info
}

/// Display tide information for one station on a particular day.
#[derive(Args, Clone, Debug)]
struct TidesArgs {
    /// ID of the desired tidal station.
    #[arg(short, long)]
    station: StationId,
}
