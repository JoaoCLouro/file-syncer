use file_syncer::cli::parse_args;
use file_syncer::types::SyncerError;

fn main () -> Result<(), SyncerError> {
    let _config = parse_args()?;


    Ok(())
}