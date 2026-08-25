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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    // Helper to spin up isolated temp directories
    fn tmp_root() -> TempDir {
        tempdir().expect("failed to create a temporary directory")
    }

    #[test]
    fn test_compute_dest_path_success() {
        let source_root = tmp_root();
        let dest_root = tmp_root();

        let source = source_root.path();
        let dest = dest_root.path();

        let file_pref = PathBuf::from("project/testable");
        let file_path = source.join(&file_pref);

        let computed = compute_dest_path(&file_path, source, dest)
            .expect("Failed to calculate path");
        assert_eq!(computed, dest.join(&file_pref));
    }

    #[test]
    fn test_compute_dest_path_mismatched_prefix_fails() {
        let source_root = tmp_root();
        let dest_root = tmp_root();

        let source = source_root.path();
        let dest = dest_root.path();

        let tmp_path = PathBuf::from("/unmatchable_path/file/");

        assert!(matches!(
            compute_dest_path(&tmp_path, source, dest),
            Err(SyncerError::ValidationError(_))
        ));
    }

    #[test]
    fn test_process_event_file_creation() {
        let source_root = tmp_root();
        let dest_root = tmp_root();
        let source = source_root.path();
        let dest = dest_root.path();

        // Arrange: Create a source file
        let src_file = source.join("hello.txt");
        std::fs::write(&src_file, b"hello rust sync").unwrap();

        // Act: Pass the Created event to your processor
        let event = SyncEvent::Created(src_file);
        process_event(&event, source, dest, false, false).expect("process_event failed");

        // Assert: Verify destination file was successfully copied
        let dest_file = dest.join("hello.txt");
        assert!(dest_file.exists());
        assert_eq!(std::fs::read_to_string(&dest_file).unwrap(), "hello rust sync");
    }

    #[test]
    fn test_process_event_nested_directory_creation() {
        let source_root = tmp_root();
        let dest_root = tmp_root();
        let source = source_root.path();
        let dest = dest_root.path();

        // Arrange: Create a deeply nested file path in source
        let nested_dir = source.join("a/b/c");
        std::fs::create_dir_all(&nested_dir).unwrap();
        let src_file = nested_dir.join("deep.txt");
        std::fs::write(&src_file, b"deep content").unwrap();

        // Act: Pass the Modified event to your processor
        let event = SyncEvent::Modified(src_file);
        process_event(&event, source, dest, false, false).expect("process_event failed");

        // Assert: Verify nested structure was successfully mirrored
        let dest_file = dest.join("a/b/c/deep.txt");
        assert!(dest_file.exists());
        assert_eq!(std::fs::read_to_string(&dest_file).unwrap(), "deep content");
    }

    #[test]
    fn test_process_event_file_deletion() {
        let source_root = tmp_root();
        let dest_root = tmp_root();
        let source = source_root.path();
        let dest = dest_root.path();

        // Arrange: File exists in both source and destination
        let src_file = source.join("remove_me.txt");
        let dest_file = dest.join("remove_me.txt");
        std::fs::write(&src_file, b"to be deleted").unwrap();
        std::fs::write(&dest_file, b"to be deleted").unwrap();

        // Act: Pass the Deleted event to your processor
        let event = SyncEvent::Deleted(src_file);
        process_event(&event, source, dest, false, false).expect("process_event failed");

        // Assert: Verify the destination file is completely removed
        assert!(!dest_file.exists(), "Destination file should have been deleted");
    }

    #[test]
    fn test_process_event_dry_run_does_not_modify_disk() {
        let source_root = tmp_root();
        let dest_root = tmp_root();
        let source = source_root.path();
        let dest = dest_root.path();

        // Arrange: Create a source file
        let src_file = source.join("dry_run.txt");
        std::fs::write(&src_file, b"should not copy").unwrap();

        // Act: Pass Created event, but with dry_run = true
        let event = SyncEvent::Created(src_file);
        // Signature is: event, source_root, dest_root, verbose, dry_run
        process_event(&event, source, dest, false, true).expect("process_event failed");

        // Assert: The destination file should NOT exist because it was a dry run
        let dest_file = dest.join("dry_run.txt");
        assert!(!dest_file.exists(), "File should not be copied during a dry run");
    }
}