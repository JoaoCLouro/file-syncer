/// Module for managing the state of the syncer application, including database initialization, retrieval, saving, and removal of file metadata.
/// This module provides functionality to interact with a sled database for storing and retrieving file metadata, which is essential for tracking the state of files during synchronization.
/// The state management is designed to ensure data integrity and efficient access to file metadata, allowing the syncer application to maintain a consistent view of the file system across synchronization operations.

use std::path::Path;
use crate::types::{FileMetaData, SyncerError};
use rkyv::{Deserialize as RkyvDeserialize, Infallible};

pub fn init_db(path: &Path) -> Result<sled::Db, SyncerError> {
    // Create a hidden the database directory if it doesn't exist
    let db_path = path.join(".syncer_state");
    std::fs::create_dir_all(&db_path).map_err(|e| SyncerError::DatabaseError(e.to_string()))?;
    // Open the sled database at the specified path
    let db = sled::open(&db_path).map_err(|e| SyncerError::DatabaseError(e.to_string()))?;
    Ok(db)
}

pub fn get_historical_state(db: &sled::Db, relative_path: &str) -> Result<Option<FileMetaData>, SyncerError> {
    // Path 2 byte conversion for sled key
    let b_slice: &[u8] = relative_path.as_bytes();
    
    match db.get(b_slice) {
        Ok(Some(value)) => {
            // Validate the raw bytes 
            let archived = rkyv::check_archived_root::<FileMetaData>(&value)
                .map_err(|e| {SyncerError::DatabaseError(format!("Corrupted database bytes: {}", e))
            })?;
            
            // Extract into owned FileMetaData struct
            let metadata: FileMetaData = archived.deserialize(&mut Infallible).unwrap();
            Ok(Some(metadata))
        },
        Ok(None) => Ok(None),
        Err(e) => Err(SyncerError::DatabaseError(e.to_string())),
    }
}

pub fn save_state(db: &sled::Db, metadata: &FileMetaData) -> Result<(), SyncerError> {
    // Serializes the FileMetaData struct into bytes and saves it to the sled database
    let serialized = rkyv::to_bytes::<_, 256>(metadata)
        .map_err(|e| SyncerError::DatabaseError(format!("Failed to serialize FileMetaData: {}", e)))?;
    // Save the serialized bytes to the database using the relative path as the key
    db.insert(metadata.relative_path.as_bytes(), serialized.as_ref())
        .map_err(|e| SyncerError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn remove_state(db: &sled::Db, relative_path: &str) -> Result<(), SyncerError> {
    db.remove(relative_path.as_bytes())
        .map_err(|e| SyncerError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn flush_db(db: &sled::Db) -> Result<(), SyncerError> {
    db.flush().map_err(|e| SyncerError::DatabaseError(e.to_string()))?;
    Ok(())
}