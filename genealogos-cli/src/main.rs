use std::error::Error;
use std::fs;
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

    /// Backend to use for Nix evaluation tracing
    #[arg(short, long, default_value = "nixtract")]
    backend: genealogos::backend::Backend,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    let json_out = genealogos(args.backend, genealogos::Source::TraceFile(args.file))?;

    // Write the CycloneDX JSON to either the specified output file or stdout
    match args.output_file {
        Some(path) => fs::write(path.into_os_string(), json_out)?,
        None => println!("{}", json_out),
    }

    Ok(())
}
