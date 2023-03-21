use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let tides = fs::read_to_string("./reference/tides.json")?;
    let tides = tidescli::predictions_from_json(&tides)?;
    println!("{:#?}", tides);

    let stations = fs::read_to_string("./reference/stations.json")?;
    let stations = tidescli::stations_from_json(&stations)?;
    println!("{:#?}", stations);

    for s in stations {
        println!("{}\t{}", s.id, s.name);
    }

    Ok(())
}
