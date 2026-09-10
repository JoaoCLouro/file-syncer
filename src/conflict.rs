use std::path::{Path, PathBuf};
use std::io::{self, Write};
use crate::types::{ConflictStrategy, FileMetaData, SyncAction, SyncerError};

pub fn resolve_conflict(source_file: &FileMetaData, dest_file: &FileMetaData, strategy: &ConflictStrategy) -> SyncAction {
    let path = PathBuf::from(&source_file.relative_path);
    
    match strategy {
        ConflictStrategy::NewerWins => {
            if source_file.modified_time >= dest_file.modified_time {
                SyncAction::CopySourceToDest(path)
            } else {
                SyncAction::CopyDestToSource(path)
            }
        },
        ConflictStrategy::SourceWins => {
            SyncAction::CopySourceToDest(path)
        },
        ConflictStrategy::DestWins => {
            SyncAction::CopyDestToSource(path)
        },
        ConflictStrategy::ManualPrompt => {
            // Your enum expects (Source path, Destination path). 
            // Passing the relative path twice allows the executor to append the root directories later.
            SyncAction::ManualResolution(path.clone(), path)
        }
    }
}

pub fn handle_manual_resolution(relative_path: &Path) -> Result<Option<SyncAction>, SyncerError> {
    let path_display = relative_path.display();
    
    loop {
        print!("Conflict detected: '{}'.\nKeep (s)ource, keep (d)estination, or s(k)ip? [s/d/k]: ", path_display);
        
        // Flush stdout to ensure the prompt prints before waiting for input
        io::stdout().flush().map_err(SyncerError::Io)?; 
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(SyncerError::Io)?;
        
        match input.trim().to_lowercase().as_str() {
            "s" | "source" => return Ok(Some(SyncAction::CopySourceToDest(relative_path.to_path_buf()))),
            "d" | "dest" => return Ok(Some(SyncAction::CopyDestToSource(relative_path.to_path_buf()))),
            "k" | "skip" => {
                println!("Skipping '{}'...", path_display);
                return Ok(None);
            },
            _ => {
                println!("Invalid input. Please enter 's', 'd', or 'k'.\n");
                continue; // Loop back and ask again
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FileMetaData, ConflictStrategy, SyncAction};
    use std::path::PathBuf;

    // Helper to quickly spawn metadata payloads in memory
    fn mock_metadata(time: u64) -> FileMetaData {
        FileMetaData {
            relative_path: "code/main.rs".to_string(),
            size: 1024,
            modified_time: time,
            hash: "dummy_hash".to_string(),
        }
    }

    #[test]
    fn test_strategy_newer_wins() {
        let older_file = mock_metadata(1000); // Created at 1000 seconds
        let newer_file = mock_metadata(5000); // Created at 5000 seconds
        let expected_path = PathBuf::from("code/main.rs");

        // Test 1: Destination has the newer file -> Engine must copy Dest to Source
        let action1 = resolve_conflict(&older_file, &newer_file, &ConflictStrategy::NewerWins);
        assert_eq!(action1, SyncAction::CopyDestToSource(expected_path.clone()));

        // Test 2: Source has the newer file -> Engine must copy Source to Dest
        let action2 = resolve_conflict(&newer_file, &older_file, &ConflictStrategy::NewerWins);
        assert_eq!(action2, SyncAction::CopySourceToDest(expected_path.clone()));
    }

    #[test]
    fn test_strategy_hard_overrides() {
        let source_file = mock_metadata(1000);
        let dest_file = mock_metadata(5000);
        let expected_path = PathBuf::from("code/main.rs");

        // Even though dest is newer, SourceWins must force a SourceToDest copy
        let source_action = resolve_conflict(&source_file, &dest_file, &ConflictStrategy::SourceWins);
        assert_eq!(source_action, SyncAction::CopySourceToDest(expected_path.clone()));

        // Even though source is newer (swapped), DestWins must force a DestToSource copy
        let dest_action = resolve_conflict(&dest_file, &source_file, &ConflictStrategy::DestWins);
        assert_eq!(dest_action, SyncAction::CopyDestToSource(expected_path));
    }
}