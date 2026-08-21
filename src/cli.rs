use crate::types::{self, Command, Config, SyncerError};
use clap::{Parser};


#[derive(Parser)]
#[command(name = "syncer")]
#[command(about = "A local one-way file synchronization engine", long_about = None)]
struct Cli {
    // Event log
    #[command(subcommand)]
    event: types::Command,
}

pub fn parse_args() -> Result<Config, SyncerError> {
    let cli = Cli::parse();

    match cli.event {
        Command::Watch {source, destination, verbose, dry_run, debounce: _,}=> {
            if verbose {
            println!("Starting watcher...");
            println!("Source: {:?}", source);
            println!("Destination: {:?}", destination);
            }

            if dry_run {
                println!("[DRY RUN MODE ENABLED]");
            }

            if !source.exists() {
                return Err(SyncerError::ValidationError(format!("Source path does not exist: {:?}", source)));
            }

            if !destination.exists() {
                return Err(SyncerError::ValidationError(format!("Destination path does not exist: {:?}", destination)));
            }

            // If both checks pass, return the config directly
            Ok(Config { source, destination })
        },

        // If any more events added match them here
    }
}