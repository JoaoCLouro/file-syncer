use std::{path::PathBuf, time::SystemTime};
use clap::Subcommand;
use thiserror::Error;
use serde::Deserialize;

// Our domain-specific event, abstracting away the notify crate's complex events
#[derive(Debug)]
pub enum SyncEvent {
    TriggerScan,
    Stop,
}

/// Newly added precise sync action 
#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    CopySourceToDest(PathBuf),
    CopyDestToSource(PathBuf),
    DeleteSource(PathBuf),
    DeleteDest(PathBuf),
    Conflict(PathBuf, SystemTime, SystemTime), // Path, source modified time, destination modified time
}


#[derive(Debug, Clone, PartialEq)]
pub struct FileMetaData {
    pub relative_path: PathBuf,
    pub size: u64,
    pub modified_time: SystemTime,
    pub hash: String,   // Hash of the file content for conflict detection
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ConflictStrategy {
    #[serde(rename = "newer")]
    NewerWins,
    #[serde(rename = "manual")]
    ManualPrompt,
    #[serde(rename = "source")]
    SourceWins,
    #[serde(rename = "dest")]
    DestWins,
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

    // new error type for access hashing
    #[error("{0}")]
    HashError(String),
    
    // new error type for conflict detection
    #[error("{0}")]
    ConflictError(String),

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

    }
}
