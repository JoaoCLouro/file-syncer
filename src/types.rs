use clap::Subcommand;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::Deserialize;
use thiserror::Error;

use std::path::PathBuf;

#[derive(Debug)]
pub enum SyncEvent {
    TriggerScan,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    CopySourceToDest(PathBuf),
    CopyDestToSource(PathBuf),
    DeleteSource(PathBuf),
    DeleteDest(PathBuf),
    ManualResolution(PathBuf, PathBuf), // Source path, Destination path
    Conflict(PathBuf, u64, u64),        // Path, source modified time, dest modified time
}

#[derive(Debug, Clone, Hash, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct FileMetaData {
    pub relative_path: String, // PathBuf conversion to string for serialization
    pub size: u64,
    pub modified_time: u64, // SystemTime converted to u64 for serialization
    pub hash: String,
}

#[derive(Debug, Clone, Hash, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Alteration {
    pub file: String,
    pub line: u16,
    pub content: String,
}

#[derive(Debug, Clone, Hash, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct EventLog {
    pub fmt: FileMetaData,
    pub altered_content: Vec<Alteration>,
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

#[derive(Error, Debug)]
pub enum SyncerError {
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Watch(String),

    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    ValidationError(String),

    #[error("{0}")]
    UnsupportedCommand(String),

    #[error("{0}")]
    HashError(String),

    #[error("{0}")]
    ConflictError(String),

    #[error("Database Error: {0}")]
    DatabaseError(String),

    #[error("Program Stopped")]
    Stop(()),
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Watch {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long, default_value = "config.toml")]
        config_path: PathBuf,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "500")]
        debounce: u64,
    },

    // New feats
    StopSpecific {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long, default_value = "config.toml")]
        config_path: PathBuf,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "500")]
        debounce: u64,
    },

    StopAllFrom {
        source: PathBuf,
        #[arg(long, default_value = "config.toml")]
        config_path: PathBuf,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "500")]
        debounce: u64,
    },

    StopAllTo {
        destination: PathBuf,
        #[arg(long, default_value = "config.toml")]
        config_path: PathBuf,
        #[arg(short, long)]
        verbose: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "500")]
        debounce: u64,
    },
}
