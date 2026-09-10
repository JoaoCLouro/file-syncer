use std::time::Duration;
use file_syncer::{config, state, index, planner, executor, watcher::DirectoryWatcher, types::{Command, SyncerError}};

fn main() -> Result<(), SyncerError> {
    let cli_command = config::parse_args()?;

    match cli_command.clone() {
        Command::Watch { source: _, destination: _, config_path, verbose: _, dry_run: _, debounce } => {
            let toml_config = config::load_toml_config(&config_path)?;
            let runtime = config::build_runtime(cli_command, toml_config);

            let db = state::init_db(&runtime.source)?;

            let (_watcher_handle, rx) = DirectoryWatcher::spawn(
                runtime.source.clone(), 
                runtime.ignore_patterns.clone()
            )?;

            if runtime.verbose {
                println!("Daemon active. Watching {} for changes...", runtime.source.display());
            }

            let debounce_duration = Duration::from_millis(debounce);

            // Defines the sync cycle as a closure so we can call it on startup AND on events
            let run_sync_cycle = || -> Result<(), SyncerError> {
                if runtime.verbose { println!("Starting reconciliation cycle..."); }
                
                let source_map = index::scan_directory_tree(&runtime.source, &runtime.ignore_patterns)?;
                let dest_map = index::scan_directory_tree(&runtime.destination, &runtime.ignore_patterns)?;
                let plan = planner::generate_sync_plan(&source_map, &dest_map, &db, &runtime.conflict_strategy)?;

                if !plan.is_empty() {
                    if runtime.dry_run {
                        println!("DRY RUN: {} pending operations.", plan.len());
                        // Assuming SyncAction derives Debug
                        for action in plan { println!(" - {:?}", action); } 
                    } else {
                        executor::execute_plan(plan, &runtime.source, &runtime.destination, &db)?;
                    }
                } else if runtime.verbose {
                    println!("Directories are fully synchronized.");
                }
                
                Ok(())
            };

            // Run once at startup to catch pre-existing differences
            if let Err(e) = run_sync_cycle() {
                eprintln!("Error during initial sync: {}", e);
            }

            // Resilient Event Consumer Loop
            while let Ok(_first_event) = rx.recv() {
                while let Ok(_) = rx.recv_timeout(debounce_duration) {
                    continue; // Drain rapid-fire events
                }

                if runtime.verbose { println!("FS mutation detected."); }

                // Catch errors locally to prevent the daemon from panicking
                if let Err(e) = run_sync_cycle() {
                    eprintln!("Daemon Error during cycle: {}. Resuming watch...", e);
                }
            }
        }
    }

    Ok(())
}