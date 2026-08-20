use std::path::PathBuf;

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
pub enum SyncerError {
Io(std::io::Error),
Watch(notify::Error),
Config(String),
}