use std::path::{PathBuf};
use clap::Subcommand;

// Represents the parsed command line arguments
pub struct Config {
    pub source: PathBuf,
    pub destination: PathBuf,
}

// Our domain-specific event, abstracting away the notify crate's complex events
pub enum SyncEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

// A unified error type for the application
#[derive(Debug)]
pub enum SyncerError {
    Io(std::io::Error),
    Watch(notify::Error),
    Config(String),
    ValidationError(String),
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Watch {
        source: PathBuf,
        destination: PathBuf,

        // verbose logging enabler
        #[arg(short, long)]
        verbose: bool,

        // Sim the sync without file changes (mainly for debug)
        #[arg(long)]
        dry_run: bool,

        // Debounce window in milliseconds
        #[arg(long, default_value = "500")]
        debounce: u64,

        // Add another needed flags later
    }
}