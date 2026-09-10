use std::{fs, path::{Path, PathBuf}};
use sled::Db;

use crate::conflict::handle_manual_resolution;

use crate::{types::{FileMetaData, SyncAction, SyncerError}, index::compute_file_hash};
use crate::state;

pub fn execute_plan(plan: Vec<SyncAction>, source_root: &Path, dest_root: &Path, db: &Db) -> Result<(), SyncerError> {
    let mut deferred_conflicts = Vec::new();

    for event in plan {
        if let SyncAction::ManualResolution(rel_path, _) = event {
            // Stash conflicts for later
            deferred_conflicts.push(rel_path); 
        } else {
            execute_action(&event, source_root, dest_root, db)?;
        }
    }
    state::flush_db(db)?;

    if !deferred_conflicts.is_empty() {
        println!("\n--- Manual Resolutions Required ({}) ---", deferred_conflicts.len());
        for rel_path in deferred_conflicts {
            if let Some(resolved_action) = handle_manual_resolution(&rel_path)? {
                execute_action(&resolved_action, source_root, dest_root, db)?;
                crate::state::flush_db(db)?; // Flush after each manual choice
            }
        }
    }
    Ok(())
}

/// Extracted into a helper so we can easily recurse for ManualResolution
fn execute_action(action: &SyncAction, source_root: &Path, dest_root: &Path, db: &Db) -> Result<(), SyncerError> {
    match action {
        SyncAction::CopySourceToDest(relative_path) => {
            let source_path = source_root.join(relative_path);
            let dest_path = dest_root.join(relative_path);
            
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(SyncerError::Io)?;
            }
            fs::copy(&source_path, &dest_path).map_err(SyncerError::Io)?;
            
            // Generate fresh metadata and save to db
            update_db_state(db, &dest_path, relative_path)?;
        },
        
        SyncAction::CopyDestToSource(relative_path) => {
            let source_path = source_root.join(relative_path);
            let dest_path = dest_root.join(relative_path);
            
            if let Some(parent) = source_path.parent() {
                fs::create_dir_all(parent).map_err(SyncerError::Io)?;
            }
            fs::copy(&dest_path, &source_path).map_err(SyncerError::Io)?;
            
            // Generate fresh metadata and save to db
            update_db_state(db, &source_path, relative_path)?;
        },
        
        SyncAction::DeleteSource(relative_path) => {
            let source_path = source_root.join(relative_path);
            if source_path.exists() {
                fs::remove_file(&source_path).map_err(SyncerError::Io)?;
            }
            // Remove from the historical B-Tree
            state::remove_state(db, &relative_path.to_string_lossy())?;
        },
        
        SyncAction::DeleteDest(relative_path) => {
            let dest_path = dest_root.join(relative_path);
            if dest_path.exists() {
                fs::remove_file(&dest_path).map_err(SyncerError::Io)?;
            }
            // Remove from the historical B-Tree
            state::remove_state(db, &relative_path.to_string_lossy())?;
        },
        
        SyncAction::ManualResolution(relative_path, _) => {
            // Get the user's choice...
            let resolved_action = handle_manual_resolution(relative_path)?;
            // ...and immediately execute it recursively!
            if let Some(resolved_action) = resolved_action {
                execute_action(&resolved_action, source_root, dest_root, db)?;
            }
        },
        
        SyncAction::Conflict(_, _, _) => {
            // As designed in your planner.rs, this is technically unreachable 
            // because resolve_conflict translates conflicts into Copies or ManualPrompts.
            unreachable!("Conflicts should be resolved by the planner before execution");
        },
    }
    Ok(())
}

/// Helper to rebuild the FileMetaData payload after a successful copy and commit it to sled.
fn update_db_state(db: &Db, absolute_path: &Path, relative_path: &PathBuf) -> Result<(), SyncerError> {
    let metadata = fs::metadata(absolute_path).map_err(SyncerError::Io)?;
    let modified = metadata.modified().map_err(SyncerError::Io)?;
    let hash = compute_file_hash(absolute_path)?;
    
    let file_meta = FileMetaData {
        relative_path: relative_path.to_string_lossy().into_owned(),
        size: metadata.len(),
        modified_time: modified.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        hash,
    };
    
    state::save_state(db, &file_meta)?;
    Ok(())
}