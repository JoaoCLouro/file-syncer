use std::{path::{Path, PathBuf}, fs};
use crate::types::{SyncEvent, SyncerError::{self, Stop}};

pub fn process_event(event: &SyncEvent, source_root: &Path, dest_root: &Path, verbose: bool, dry_run: bool) -> Result<(), SyncerError> {
    match event {
        SyncEvent::Created(path) | SyncEvent::Modified(path) => {
            let dest_path = compute_dest_path(&path, &source_root, &dest_root)?;
            // Ensure parent dirs exist if any
            if let Some(parent) = dest_path.parent() {
                if verbose {println!("Creating needed directories to reach {}", dest_path.display());}
                std::fs::create_dir_all(parent)?;
            }
            if !dry_run {fs::copy(&path, &dest_path)?;}
            if verbose {println!("File at {} created!", dest_path.display());}
        },

        SyncEvent::Deleted(path) => {
            let dest_path = compute_dest_path(&path, &source_root, &dest_root)?;
            
            // Nothing to remove if the destination file does not exist.
            if !dest_path.exists() {
                if verbose {println!("No match for {} found. Nothing to remove!", dest_path.display());}
                return Ok(());
            }
            if !dry_run {fs::remove_file(&dest_path)?;}
            if verbose {println!("File at {} removed!", dest_path.display());}

        },

        SyncEvent::Stop => {
            // Triggers the program stop (Not an actual error)
            return Err(Stop(()))
        }
    }
    Ok(())
}

fn compute_dest_path(src_file: &Path, src_root: &Path, dest_root: &Path) -> Result<PathBuf, SyncerError> {
    let rel_path = src_file.strip_prefix(src_root).map_err(|_| {
            SyncerError::ValidationError(format!("File path is not inside source root: {:?}", src_file))

    })?;
    Ok(dest_root.join(rel_path))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use tempfile::{TempDir, tempdir};

    fn tmp_root() -> TempDir {
        tempdir().expect("failed to create a temporary directory")
    }

    #[test]
    fn test_compute_dest_path_success() {
        // Base directories to strip 
        let source_root = tmp_root();
        let dest_root = tmp_root();

        let source = source_root.path();
        let dest = dest_root.path();

        // Prefix and full path to assert
        let file_pref = PathBuf::from("project/testable");
        let src_file_path: PathBuf = source.join(&file_pref);
        let dest_file_path: PathBuf = dest.join(&file_pref);

        let mut computed = compute_dest_path(&src_file_path, source, dest).expect("Failed to calculate path");
        assert_eq!(computed, dest.join(&file_pref));

        computed = compute_dest_path(&dest_file_path, dest, source).expect("Failed to calculate path");
        assert_eq!(computed, source.join(&file_pref));
    }

    #[test]
    fn test_compute_dest_path_mismatched_prefix_fails() {
        // Base directories to strip 
        let source_root = tmp_root();
        let dest_root = tmp_root();

        let source = source_root.path();
        let dest = dest_root.path();

        // Dir to mismatch
        let tmp_path = PathBuf::from("/unmatchable_path/file/");

        assert!(matches!(
            compute_dest_path(&tmp_path, source, dest),
            Err(SyncerError::ValidationError(_))
        ));
        assert!(matches!(
            compute_dest_path(&tmp_path, dest, source),
            Err(SyncerError::ValidationError(_))
        ));
    }
}
