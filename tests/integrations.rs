use std::fs;
use tempfile::tempdir;
use file_syncer::{state, index, planner, executor, types::ConflictStrategy};

#[test]
fn test_full_synchronization_pipeline() {
    // 1. Setup Isolated Temporary Directories
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let source_dir = temp_dir.path().join("source");
    let dest_dir = temp_dir.path().join("dest");
    
    fs::create_dir(&source_dir).unwrap();
    fs::create_dir(&dest_dir).unwrap();

    // 2. Populate Initial State
    // We put one file in source, one in dest to test bidirectional syncing
    fs::write(source_dir.join("source_file.txt"), "A").unwrap();
    fs::write(dest_dir.join("dest_file.txt"), "B").unwrap();

    // 3. Mount Database
    let db = state::init_db(&source_dir).expect("Failed to init db");
    let strategy = ConflictStrategy::NewerWins;
    let ignore_patterns = vec![];

    // ==========================================
    // CYCLE 1: Bidirectional Copying
    // ==========================================
    let source_map = index::scan_directory_tree(&source_dir, &ignore_patterns).unwrap();
    let dest_map = index::scan_directory_tree(&dest_dir, &ignore_patterns).unwrap();
    
    let plan = planner::generate_sync_plan(&source_map, &dest_map, &db, &strategy).unwrap();
    
    // The plan should copy source_file.txt to dest, and dest_file.txt to source
    assert_eq!(plan.len(), 2, "Expected exactly 2 sync actions");
    
    executor::execute_plan(plan, &source_dir, &dest_dir, &db).unwrap();

    // Verify Cycle 1 physical state
    assert!(dest_dir.join("source_file.txt").exists(), "Source file was not copied to dest");
    assert!(source_dir.join("dest_file.txt").exists(), "Dest file was not copied to source");

    // ==========================================
    // CYCLE 2: State-Aware Deletion
    // ==========================================
    // We simulate the user deleting 'source_file.txt' from the source directory
    fs::remove_file(source_dir.join("source_file.txt")).unwrap();

    let source_map_2 = index::scan_directory_tree(&source_dir, &ignore_patterns).unwrap();
    let dest_map_2 = index::scan_directory_tree(&dest_dir, &ignore_patterns).unwrap();
    
    let plan_2 = planner::generate_sync_plan(&source_map_2, &dest_map_2, &db, &strategy).unwrap();
    
    // Because the file is in dest_map but missing from source_map, AND has a history in the DB,
    // the planner must queue a DeleteDest action.
    assert_eq!(plan_2.len(), 1, "Expected exactly 1 deletion action");
    
    executor::execute_plan(plan_2, &source_dir, &dest_dir, &db).unwrap();

    // Verify Cycle 2 physical state
    assert!(!dest_dir.join("source_file.txt").exists(), "Deleted file was not mirrored to dest");
}