use std::{path::{Path, PathBuf}, fs};
use crate::types::{SyncEvent, SyncerError};

pub fn process_event(event: &SyncEvent, source_root: &Path, dest_root: &Path, verbose: bool) -> Result<(), SyncerError> {
    match event {
        SyncEvent::Created(path) | SyncEvent::Modified(path) => {
            let dest_path = compute_dest_path(&path, &source_root, &dest_root)?;
            // Ensure parent dirs exist if any
            if let Some(parent) = dest_path.parent() {
                if verbose {println!("Creating needed directories to reach {}", dest_path.display());}
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&path, &dest_path)?;
            if verbose {println!("File at {} created!", dest_path.display());}
        }

        SyncEvent::Deleted(path) => {
            let dest_path = compute_dest_path(&path, &source_root, &dest_root)?;
            
            // Ensure file exits
            if dest_path.exists() {
                std::fs::remove_file(&dest_path)?;
                if verbose {println!("File at {} removed!", dest_path.display());}
            }
            else if verbose {println!("No match for {} found. Nothing to remove!", dest_path.display());}
        }
    }
    Ok(())
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
