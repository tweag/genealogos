use std::error::Error;
use std::fs;
use std::path;

use clap::Parser;

use genealogos::cyclonedx;
use genealogos::genealogos;

/// `cli` application for processing data files and generating CycloneDX output
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the input nixtract file
    #[arg(short, long, required_unless_present = "flake_ref")]
    file: Option<path::PathBuf>,

    /// Flake reference (e.g. `nixpkgs`)
    #[arg(long, required_unless_present = "file")]
    flake_ref: Option<String>,

    /// Attribute path (e.g. `hello`)
    #[arg(long, required_unless_present = "file")]
    attribute_path: Option<String>,

    /// Optional path to the output CycloneDX file (default: stdout)
    output_file: Option<path::PathBuf>,

    /// Backend to use for Nix evaluation tracing
    #[arg(long, default_value = "nixtract")]
    backend: genealogos::backend::Backend,

    /// Optional CycloneDX version to use
    #[arg(long, default_value = "1.5")]
    cyclonedx_version: cyclonedx::Version,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // If a file was specified, use that as the input file as the Source, otherwise use the flake reference and attribute path
    let source = if let Some(file) = args.file {
        genealogos::Source::TraceFile(file)
    } else {
        genealogos::Source::Flake {
            flake_ref: args.flake_ref.unwrap(),
            attribute_path: args.attribute_path,
        }
    };

    // Generate the CycloneDX output
    let output = genealogos(args.backend, source, args.cyclonedx_version)?;

    // Write the output to the specified file, or stdout if no file was specified
    if let Some(output_file) = args.output_file {
        fs::write(output_file, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}
