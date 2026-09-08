use std::path::{Path, PathBuf};
use std::fs;
use clap::Parser;
use serde::Deserialize;
use crate::types::{Command, ConflictStrategy, SyncerError};



// ====================================================
// Structs and functions for configuration management
// ====================================================

/// Command Line Interface (CLI) configuration for the syncer application.
#[derive(Parser)]
#[command(name = "syncer")]
#[command(about = "A local one-way file synchronization engine", long_about = None)]
struct Cli {
    // Event log
    #[command(subcommand)]
    event: Command,
}

/// Configuration loaded from a TOML file.
#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    #[serde(default = "default_conflict_strategy")]
    pub conflict_strategy: ConflictStrategy,
    
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

/// Returns the default conflict strategy.
/// Update this function to change the default conflict resolution behavior.
fn default_conflict_strategy() -> ConflictStrategy {
    ConflictStrategy::NewerWins
}

impl Default for TomlConfig {
    fn default() -> Self {
        Self {
            conflict_strategy: default_conflict_strategy(),
            ignore_patterns: Vec::new(),
        }
    }
}


// ====================================================================
// Functions for parsing CLI arguments and loading TOML configuration
// ====================================================================

pub fn parse_args() -> Result<Command, SyncerError> {
    let cli = Cli::parse();

    match cli.event {
        Command::Watch {source, destination, verbose, dry_run, debounce,}=> {
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
            Ok(Command::Watch { source, destination, verbose, dry_run, debounce})
        },

        // If any more events added match them here
    }
}

pub fn load_toml_config(path: &Path) -> Result<TomlConfig, SyncerError> {
    let toml_str = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File is missing: return silent default 
            return Ok(TomlConfig::default());
        }
        Err(e) => {
            // File exists but cannot be read 
            return Err(SyncerError::Config(format!("Failed to read config file at {:?}: {}", path, e)));
        }
    };

    let config: TomlConfig = toml::from_str(&toml_str).map_err(|e| {
        SyncerError::Config(format!("Failed to parse TOML syntax: {}", e))
    })?;

    Ok(config)
}

pub fn build_runtime(cli: Command, toml: TomlConfig) -> RuntimeConfig {
    
}