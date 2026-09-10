use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use sled::Db;

use crate::types::{FileMetaData, SyncAction, ConflictStrategy, SyncerError};
use crate::state::get_historical_state;
use crate::conflict::resolve_conflict;


pub fn generate_sync_plan(source_map: &HashMap<String, FileMetaData>, dest_map: &HashMap<String, FileMetaData>, db: &Db, conflict_strategy: &ConflictStrategy) -> Result<Vec<SyncAction>, SyncerError> {
    let mut plan = Vec::new();
    let mut all_keys = HashSet::new();

    // Pool all unique relative paths from both live directories
    for key in source_map.keys() { all_keys.insert(key); }
    for key in dest_map.keys() { all_keys.insert(key); }

    for key in all_keys {
        let in_source = source_map.get(key);
        let in_dest = dest_map.get(key);
        let in_history = get_historical_state(db, key)?;

        match (in_source, in_dest) {
            // 1. COLLISION: Exists in both locations
            (Some(src_meta), Some(dest_meta)) => {
                if src_meta.hash != dest_meta.hash {
                    plan.push(resolve_conflict(src_meta, dest_meta, conflict_strategy));
                }
            }
            
            // 2. SOURCE ONLY: Exists in source, missing in destination
            (Some(_), None) => {
                if in_history.is_some() {
                    // It was synced before, meaning it was intentionally deleted from the destination
                    plan.push(SyncAction::DeleteSource(PathBuf::from(key)));
                } else {
                    // It has no history, meaning it's a brand new file created in the source
                    plan.push(SyncAction::CopySourceToDest(PathBuf::from(key)));
                }
            }
            
            // 3. DESTINATION ONLY: Exists in destination, missing in source
            (None, Some(_)) => {
                if in_history.is_some() {
                    // It was synced before, meaning it was intentionally deleted from the source
                    plan.push(SyncAction::DeleteDest(PathBuf::from(key)));
                } else {
                    // It has no history, meaning it's a brand new file created in the destination
                    plan.push(SyncAction::CopyDestToSource(PathBuf::from(key)));
                }
            }
            
            // Should be mathematically impossible since keys are derived from the maps
            (None, None) => unreachable!(),
        }
    }

    Ok(plan)
}