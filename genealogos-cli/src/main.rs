use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

use clap::Parser;

use genealogos::genealogos;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let file = File::open(args.file)?;

    let json_out = genealogos(io::BufReader::new(file).lines().flatten())?;

    println!("{}", json_out);

    Ok(())
}
