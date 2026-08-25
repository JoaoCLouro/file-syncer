use std::path::{PathBuf};
use clap::Subcommand;
use thiserror::Error;

// Our domain-specific event, abstracting away the notify crate's complex events
#[derive(Debug)]
pub enum SyncEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Stop,
}

// A unified error type for the application
#[derive(Error, Debug)]
pub enum SyncerError {
    #[error("")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Watch(String),

    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    ValidationError(String),
    
    #[error("Program Stopped")]
    Stop(())
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