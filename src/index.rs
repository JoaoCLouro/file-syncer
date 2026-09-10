// Module responsible for the indexing and hashing of the file system
// This module provides functionality to scan a directory tree, compute file hashes, and generate a mapping of file paths to their corresponding metadata. It is essential for maintaining an up-to-date index of the file system, which is used in synchronization operations.


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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_compute_file_hash_correctness() {
        // Spawn an isolated temporary file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        
        // Write exactly 11 bytes to disk
        write!(temp_file, "hello world").expect("Failed to write buffer");
        
        // The universally accepted SHA-256 hash for the string "hello world"
        let expected_sha256 = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        
        // Feed the physical file path through your 8KB chunking engine
        let computed_hash = compute_file_hash(temp_file.path()).unwrap();
        
        assert_eq!(computed_hash, expected_sha256, "Cryptographic hash mismatch");
    }
}