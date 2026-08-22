use std::{path::{Path, PathBuf}, fs};
use crate::types::{SyncEvent, SyncerError};

pub fn process_event(event: &SyncEvent, source_root: &Path, dest_root: &Path) -> Result<(), SyncerError> {
    todo!("Implement syncer logic");
}


fn copy_file(src: &Path, dest: &Path) -> Result<(), SyncerError> {
    fs::copy(src, dest)?;
    Ok(())
}


fn remove_file(target: &Path) -> Result<(), SyncerError> {
    // Tries to delete the file if possible else converts the io::Error to a SyncerError
    fs::remove_file(target).map_err(SyncerError::from)
}


fn compute_dest_path(src_file: &Path, src_root: &Path, dest_root: &Path) -> Result<PathBuf, SyncerError> {
    let rel_path = src_file.strip_prefix(src_root).map_err(|_| {
            SyncerError::ValidationError(format!("File path is not inside source root: {:?}", src_file))

    })?;
    Ok(dest_root.join(rel_path))
}
