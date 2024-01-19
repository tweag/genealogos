use std::error::Error;
use std::fs;
use std::io::{self, BufRead};
use std::path;

use clap::Parser;

use genealogos::genealogos;

/// `cli` application for processing data files and generating CycloneDX output
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the input nixtract file
    file: path::PathBuf,

    /// Optional path to the output CycloneDX file (default: stdout)
    output_file: Option<path::PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    let file = fs::File::open(args.file)?;

    // Read lines from the input file and flatten into a single iterator
    let lines = io::BufReader::new(file).lines().flatten();

    // Process the input data using `genealogos` and generate CycloneDX JSON
    let json_out = genealogos(lines)?;

    // Write the CycloneDX JSON to either the specified output file or stdout
    match args.output_file {
        Some(path) => fs::write(path.into_os_string(), json_out)?,
        None => println!("{}", json_out),
    }

    Ok(())
}
