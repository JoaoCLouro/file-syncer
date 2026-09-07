// Module responsible for the indexing and hashing of the file system
use walkdir::WalkDir;
use std::{collections::HashMap, path::{Path, PathBuf}};
use crate::types::{FileMetaData, SyncerError};

pub fn compute_file_hash(path: &Path) -> Result<String, SyncerError> {
    use std::fs::File;
    use std::io::{BufReader, Read};
    use sha2::{Sha256, Digest};

    // File reading and hashing logic
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    // Hash formatting pipeline: Convert the hash bytes to a hexadecimal string representation
    let digest = hasher.finalize();
    Ok(digest.iter().map(|byte| format!("{:02x}", byte)).collect())
}

pub fn scan_directory_tree(root: &Path, ignore_patterns: &[String]) -> Result<HashMap<PathBuf, FileMetaData>, SyncerError> {
    // Initialize a HashMap to store the file metadata
    let mut file_map = HashMap::new();

    // Initializes the WalkDir iterator to traverse the directory tree
    for entry in WalkDir::new(root).into_iter()
                                             .filter(|e| e.is_ok())
                                             // Errors got filtered out
                                             .map(|e| e.unwrap()) {
                                                let path = entry.path();

                                                // Skip directories and files that match the ignore patterns
                                                if path.is_dir() || ignore_patterns.iter().any(|pattern| path.to_string_lossy().contains(pattern)) {
                                                    continue;
                                                }

                                                // Compute the file hash and metadata
                                                match compute_file_hash(path) {
                                                    Ok(hash) => {
                                                        let metadata = std::fs::metadata(path)?;
                                                        let file_meta = FileMetaData {
                                                            relative_path: path.strip_prefix(root).unwrap().to_path_buf(),
                                                            size: metadata.len(),
                                                            modified_time: metadata.modified()?,
                                                            hash,
                                                        };
                                                        file_map.insert(file_meta.relative_path.clone(), file_meta);
                                                    },
            
                                                    Err(e) => {
                                                        eprintln!("Error computing hash for {}: {}", path.display(), e);
                                                    }   
                                                }
                                             }
    Ok(file_map)
}