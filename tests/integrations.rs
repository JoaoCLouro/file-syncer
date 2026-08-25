use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

use file_syncer::sync::process_event;
use file_syncer::watcher::start_watcher;
use file_syncer::types::SyncEvent;

#[test]
fn test_end_to_end_file_synchronization() {
    let source_dir = tempdir().expect("Failed to create source temp dir");
    let dest_dir = tempdir().expect("Failed to create dest temp dir");

    let source = source_dir.path().canonicalize().expect("Failed to canonicalize source");
    let dest = dest_dir.path().canonicalize().expect("Failed to canonicalize dest");

    let (tx, rx) = mpsc::channel::<SyncEvent>();
    let debounce_ms = 100;

    let _watcher = start_watcher(&source, tx, debounce_ms)
        .expect("Failed to start watcher in integration test");

    let src_clone = source.clone();
    let dest_clone = dest.clone();

    let sync_handle = thread::spawn(move || {
        while let Ok(event) = rx.recv_timeout(Duration::from_millis(2000)) {
            if matches!(event, SyncEvent::Stop) {
                break;
            }
            let _ = process_event(&event, &src_clone, &dest_clone, false, false);
        }
    });

    // Give inotify time to attach to the source root
    thread::sleep(Duration::from_millis(150));

    // --- Scenario A: File Creation ---
    let source_file = source.join("sync_test.txt");
    fs::write(&source_file, b"systems programming").unwrap();

    thread::sleep(Duration::from_millis(debounce_ms + 400));

    let dest_file = dest.join("sync_test.txt");
    assert!(
        dest_file.exists(),
        "Destination file was not created at {:?}",
        dest_file
    );
    assert_eq!(
        fs::read_to_string(&dest_file).unwrap(),
        "systems programming"
    );

    // --- Scenario B: File Modification ---
    fs::write(&source_file, b"updated content").unwrap();
    thread::sleep(Duration::from_millis(debounce_ms + 400));

    assert_eq!(
        fs::read_to_string(&dest_file).unwrap(),
        "updated content"
    );

    // --- Scenario C: File Deletion ---
    fs::remove_file(&source_file).unwrap();
    thread::sleep(Duration::from_millis(debounce_ms + 400));

    assert!(!dest_file.exists(), "Destination file was not deleted");

    let _ = sync_handle.join();
}