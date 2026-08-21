use file_syncer::cli::parse_args;
use file_syncer::compress;
use file_syncer::types::SyncerError;
use clap::Parser;

fn main () -> Result<(), SyncerError> {
    let config = parse_args()?;

    
    Ok(())
}