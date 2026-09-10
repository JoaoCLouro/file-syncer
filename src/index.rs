// Module responsible for the indexing and hashing of the file system
use walkdir::WalkDir;
use std::{collections::HashMap, path::{Path}};
use crate::types::{FileMetaData, SyncerError};

pub fn compute_file_hash(path: &Path) -> Result<String, SyncerError> {
    use std::fs::File;
    use std::io::{BufReader, Read};
    use sha2::{Sha256, Digest};

    // File reading and hashing logic
    let file = File::open(path).map_err(|e| SyncerError::Io(e))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer for reading the file in chunks

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| SyncerError::Io(e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Hash formatting pipeline: Convert the hash bytes to a hexadecimal string representation
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{:02x}", byte)).collect())
}

pub fn scan_directory_tree(root: &Path, ignore_patterns: &[String]) -> Result<HashMap<String, FileMetaData>, SyncerError> {
    // Initialize a HashMap to store the file metadata
    let mut file_map: HashMap<String, FileMetaData> = HashMap::new();

    // Initializes the WalkDir iterator to traverse the directory tree
    for entry in WalkDir::new(root).into_iter()
                                             .filter(|e| e.is_ok())
                                             // Errors got filtered out
                                             .map(|e| e.unwrap()) {
                                                let path = entry.path();
                                                let path_str = path.to_string_lossy().into_owned();

                                                if path.is_dir() || path_str.contains(".syncer_state") || ignore_patterns.iter().any(|pattern| path_str.contains(pattern)) {
                                                    continue;
                                                }

                                                // Compute the file hash and metadata
                                                match compute_file_hash(path) {
                                                    Ok(hash) => {
                                                        let metadata = std::fs::metadata(path).map_err(|e| SyncerError::Io(e))?;
                                                        let modified = metadata.modified().map_err(|e| SyncerError::Io(e))?;
                                                        
                                                        let file_path = path.strip_prefix(root)
                                                            .map(|p| p.to_string_lossy().into_owned())
                                                            .unwrap_or_else(|_| path_str);
                                                        
                                                        let file_meta = FileMetaData {
                                                            relative_path: file_path.clone(), // Use clone for the internal payload
                                                            size: metadata.len(),
                                                            modified_time: modified.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                                                            hash,
                                                        };
                                                        
                                                        file_map.insert(file_path, file_meta);
                                                    },
            
                                                    Err(e) => {
                                                        eprintln!("Error computing hash for {}: {}", path.display(), e);
                                                    }   
                                                }
                                             }
    Ok(file_map)
}