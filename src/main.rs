use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

use clap::Parser;
use serde_cyclonedx::cyclonedx::v_1_5 as cyclonedx;

use crate::model::Model;
use crate::nixtract::Nixtract;

mod model;
mod nixtract;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();

    let file = File::open(args.file)?;

    let mut entries = vec![];

    for line in io::BufReader::new(file).lines().flatten() {
        let entry: nixtract::NixtractEntry = serde_json::from_str(&line.trim()).unwrap();
        entries.push(entry);
    }
    let nixtract: Nixtract = Nixtract { entries };

    let model: Model = nixtract.into();
    let cyclonedx: cyclonedx::CycloneDx = model.into();
    let json_out = serde_json::to_string(&cyclonedx).unwrap();

    println!("{}", json_out);

    Ok(())
}
