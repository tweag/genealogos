use std::error::Error;
use std::fs;
use std::path;

use clap::Parser;

use genealogos::backend::BackendHandle;
use genealogos::cyclonedx;
use genealogos::json_string;

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
    backend: genealogos::backend::BackendEnum,

    /// Optional CycloneDX version to use
    #[arg(long, default_value = "1.5")]
    cyclonedx_version: cyclonedx::Version,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
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

    // Initialize the backend and get access to the status update messages
    let (backend, handle) = args.backend.get_backend();
    let messages = handle.get_messages()?;

    // Create the indicatif multi progress bar
    let multi = indicatif::MultiProgress::new();

    // Initialize the logger using the indicatif log bridge
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter_level(args.verbose.log_level_filter());
    let logger = log_builder.build();
    indicatif_log_bridge::LogWrapper::new(multi.clone(), logger)
        .try_init()
        .unwrap();

    let spinner_style =
        indicatif::ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")?
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈");

    // Start a thread to generate the CycloneDX output
    let thread_handle =
        std::thread::spawn(move || json_string(backend, source, args.cyclonedx_version));

    // Spawn a new thread that will update the TUI
    // Create a progress bar for rayon thread in the global thread pool
    let mut progress_bars = Vec::new();
    for _ in 0..handle.get_num_ids() {
        let pb = multi.add(indicatif::ProgressBar::new(0));
        pb.set_style(spinner_style.clone());
        progress_bars.push(pb);
    }

    for message in messages {
        match message.status {
            nixtract::message::Status::Started => {
                progress_bars[message.id].set_message(format!("Processing {}", message.path));
            }
            nixtract::message::Status::Completed => {
                progress_bars[message.id].set_message(format!("Processed {}", message.path));
                progress_bars[message.id].inc(1);
            }
            nixtract::message::Status::Skipped => {
                progress_bars[message.id].set_message(format!("Skipped {}", message.path));
            }
        }
    }

    for pb in progress_bars {
        pb.finish();
    }

    multi.clear().expect("Failed to clear the progress bar");

    let output = thread_handle
        .join()
        .expect("Failed to join the generation thread")?;

    // Write the output to the specified file, or stdout if no file was specified
    if let Some(output_file) = args.output_file {
        fs::write(output_file, output)?;
    } else {
        println!("{}", output);
    }

    Ok(())
}
