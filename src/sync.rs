use std::path::{Path, PathBuf};
use crate::types::{SyncEvent, SyncerError};

pub fn process_event(event: &SyncEvent, source_root: &Path, dest_root: &Path) -> Result<(), SyncerError> {
    todo!("Implement syncer logic");
}


fn copy_file(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    todo!("Implement copy file logic");
}


fn remove_file(target: &Path) -> Result<(), std::io::Error> {
    todo!("Implement remove file logic");
}


fn compute_dest_path(src_file: &Path, src_root: &Path, dest_root: &Path) -> PathBuf {
    todo!("Implement path computation logic");
}
