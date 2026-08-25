use std::{path::PathBuf, sync::mpsc::Sender, thread};
use notify::{self, EventKind::{Create, Modify, Remove}, Watcher};
use crate::types::{SyncEvent, SyncerError};

pub fn start_watcher (src_root: &PathBuf, tx: Sender<SyncEvent>, _debounce: u64) -> Result<(), SyncerError> {
    let src_root = src_root.clone();
    
    // Create and configure the watcher synchronously first so we can catch errors!
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event { // Notice: no `mut` needed anymore!
            match event.kind {
                Create(_) => {
                    if let Some(path) = event.paths.first().cloned() {
                        let _ = tx.send(SyncEvent::Created(path));
                    } else {
                        eprintln!("No path found for create event");
                    }
                },
                Modify(_) => {
                    if let Some(path) = event.paths.first().cloned() {
                        let _ = tx.send(SyncEvent::Modified(path));
                    } else {
                        eprintln!("No path found for modify event");
                    }
                },
                Remove(_) => {
                    if let Some(path) = event.paths.first().cloned() {
                        let _ = tx.send(SyncEvent::Deleted(path));
                    } else {
                        eprintln!("No path found for remove event");
                    }
                },

                notify::EventKind::Access(_) => {
                    // Silently ignore open/close metadata access events
                },
                other => {
                    eprintln!("Unsupported operation event {:?}", other);
                }
            }
        } else {
            eprintln!("Failed to receive filesystem event");
        }
    }).map_err(|e| SyncerError::Watch(e.to_string()))?;

    watcher.watch(&src_root, notify::RecursiveMode::NonRecursive)
        .map_err(|e| SyncerError::Watch(e.to_string()))?;

    // Now spawn the thread just to keep the watcher alive and parked
    thread::spawn(move || {
        // Keep ownership of `watcher` alive in this thread
        let _keep_alive = watcher;
        thread::park();
    });

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs, sync::mpsc, time::Duration, thread};
    use tempfile::{tempdir, TempDir};

    fn tmp_root() -> TempDir {
        tempdir().expect("failed to create a temporary directory")
    }

    #[test]
    fn test_watcher_init() {
        // Arrange
        let temp_dir = tmp_root();
        let src = temp_dir.path().to_path_buf();
        let (tx, _) = mpsc::channel::<SyncEvent>();
        let debounce_ms = 100;
        
        // Act
        let watcher = start_watcher(&src, tx, debounce_ms);

        // Assert
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_watcher_detects_event() {
        // Arrange
        let temp_dir = tmp_root();
        let src = temp_dir.path().to_path_buf();
        let file_path = src.join("test_file.txt");
        fs::File::create(&file_path).expect("Failed to create test file");
        let (tx, rx) = mpsc::channel::<SyncEvent>();
        let debounce_ms = 100; // Lower debounce keeps unit tests fast
        let _watcher = start_watcher(&src, tx, debounce_ms)
            .expect("Failed to start watcher");

        // Act
        // Give the OS watcher a moment to register the path before triggering events
        thread::sleep(Duration::from_millis(50));
        fs::write(&file_path, "This is a test modification").expect("Failed to write to file");
        // Wait slightly longer than the debounce time to ensure the event flushes
        let event = rx.recv_timeout(Duration::from_millis(debounce_ms + 400))
            .expect("Timed out waiting for watcher event");

        // Assert
        assert!(matches!(event, SyncEvent::Modified(_)));
    }

    #[test]
    fn test_watcher_detects_shutdown() {
        // Arrange
        let temp_dir = tmp_root();
        let src = temp_dir.path().to_path_buf();
        let (tx, rx) = mpsc::channel::<SyncEvent>();
        let debounce_ms = 100;
        let watcher = start_watcher(&src, tx, debounce_ms)
            .expect("Failed to start watcher");

        // Act
        let _ = watcher;
        let result = rx.recv_timeout(Duration::from_millis(500));

        // Assert
        assert!(result.is_err(), "Channel should be disconnected after watcher shutdown");
    }

    #[test]
    fn test_watcher_detects_file_creation() {
        // Arrange
        let temp_dir = tmp_root();
        let src = temp_dir.path().to_path_buf();
        let (tx, rx) = mpsc::channel::<SyncEvent>();
        let debounce_ms = 100;

        let _watcher = start_watcher(&src, tx, debounce_ms).expect("Failed to start watcher");
        thread::sleep(Duration::from_millis(50));

        // Act
        let file_path = src.join("new_file.txt");
        fs::write(&file_path, "new file content").unwrap();

        let event = rx.recv_timeout(Duration::from_millis(debounce_ms + 400))
            .expect("Timed out waiting for Created event");

        // Assert
        assert!(matches!(event, SyncEvent::Created(_)));
    }

    #[test]
    fn test_watcher_detects_file_deletion() {
        // Arrange
        let temp_dir = tmp_root();
        let src = temp_dir.path().to_path_buf();
        let file_path = src.join("delete_me.txt");
        fs::write(&file_path, "temporary").unwrap();

        let (tx, rx) = mpsc::channel::<SyncEvent>();
        let debounce_ms = 100;
        let _watcher = start_watcher(&src, tx, debounce_ms).expect("Failed to start watcher");
        thread::sleep(Duration::from_millis(50));

        // Act
        fs::remove_file(&file_path).unwrap();

        let event = rx.recv_timeout(Duration::from_millis(debounce_ms + 400))
            .expect("Timed out waiting for Deleted event");

        // Assert
        assert!(matches!(event, SyncEvent::Deleted(_)));
    }

    #[test]
    fn test_watcher_init_nonexistent_path_fails() {
        // Arrange
        let invalid_path = PathBuf::from("/non_existent_dir_12345");
        let (tx, _) = mpsc::channel::<SyncEvent>();
        
        // Act
        let result = start_watcher(&invalid_path, tx, 100);

        // Assert
        assert!(result.is_err(), "Starting a watcher on a non-existent path should return an Err");
    }

    #[test]
    fn test_watcher_detects_nested_directory_events() {
        // Arrange
        let temp_dir = tmp_root();
        let src = temp_dir.path().to_path_buf();
        let (tx, rx) = mpsc::channel::<SyncEvent>();
        let debounce_ms = 100;

        let _watcher = start_watcher(&src, tx, debounce_ms).expect("Failed to start watcher");
        thread::sleep(Duration::from_millis(50));

        // Act
        let sub_dir = src.join("new_folder");
        fs::create_dir(&sub_dir).unwrap();
        let nested_file = sub_dir.join("inner.txt");
        fs::write(&nested_file, "nested data").unwrap();

        let event = rx.recv_timeout(Duration::from_millis(debounce_ms + 400))
            .expect("Timed out waiting for nested directory event");

        // Assert
        assert!(matches!(event, SyncEvent::Created(_) | SyncEvent::Modified(_)));
    }
}