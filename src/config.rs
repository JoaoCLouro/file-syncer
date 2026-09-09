/// Module for managing configuration of the syncer application, including command-line arguments and TOML configuration files.
/// This module provides functionality to parse command-line arguments using `clap`, load configuration from a TOML file, and combine both sources into a runtime configuration structure.
/// The configuration management is designed to be flexible, allowing users to specify settings via command-line arguments or a configuration file, with sensible defaults provided for missing values.


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
#[command(about = "A local bidirectional file synchronization engine", long_about = None)]
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

#[derive(Debug, Clone)]
/// Runtime configuration that combines CLI arguments and TOML configuration.
pub struct RuntimeConfig {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub verbose: bool,
    pub dry_run: bool,
    pub debounce: u64,
    pub conflict_strategy: ConflictStrategy,
    pub ignore_patterns: Vec<String>,
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
    match cli {
        Command::Watch { source, destination, verbose, dry_run, debounce } => {
            RuntimeConfig {
                source,
                destination,
                verbose,
                dry_run,
                debounce,
                conflict_strategy: toml.conflict_strategy,
                ignore_patterns: toml.ignore_patterns,
            }
        },
        // Match potential future commands here
    }
}