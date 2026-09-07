// Module responsible for the bidirectional synchronization planning
use std::{collections::HashMap, path::{Path, PathBuf}};
use crate::types::{FileMetaData, SyncAction, SyncerError};

pub fn compute_sync_plan(source_index: &HashMap<PathBuf, FileMetaData,>, dest_index: &HashMap<PathBuf, FileMetaData>) -> Result<Vec<SyncAction>, SyncerError> {

    let mut actions= Vec::<SyncAction>::new();

    // Source eval
    for entry in source_index.keys() {
        // File doesn't exist
        if !dest_index.contains_key(entry) {
            actions.push(SyncAction::CopySourceToDest(entry.clone()));
            continue;
        }

        let src = source_index.get(entry).expect("source entry disappeared during planning");
        let dest = dest_index.get(entry).expect("destination entry disappeared during planning");
        
        // Files content don't match        
        if src.hash != dest.hash {
            actions.push(SyncAction::Conflict(entry.to_owned(), src.modified_time, dest.modified_time));
        }
    }

    // Dest eval
    for entry in dest_index.keys() {
        // File created only on the dest side
        if !source_index.contains_key(entry) {
            actions.push(SyncAction::CopyDestToSource(entry.clone()));
        }
    }

    todo!("Create a method to eval deletions! Could use a small data base for previous hashmaps");

    Ok(actions)
}