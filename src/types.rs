use std::{path::PathBuf, time::SystemTime};
use clap::Subcommand;
use thiserror::Error;
use serde::Deserialize;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

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
    Conflict(PathBuf, SystemTime, SystemTime),
}

#[derive(Debug, Clone, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct FileMetaData {
    pub relative_path: String,  // PathBuf convertion to string for serialization
    pub size: u64,
    pub modified_time: u64,     // SystemTime converted to u64 for serialization
    pub hash: String,
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
        #[arg(short, long)]
        verbose: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long, default_value = "500")]
        debounce: u64,
    },
}