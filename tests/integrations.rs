use file_syncer::{sync, types::*};
use tempfile::tempdir;

#[test]
fn test_end_to_end_sync() {
    let src_dir = tempdir().unwrap();
    let dest_dir = tempdir().unwrap();

    // Test full file creation, modification, and deletion flow here
}