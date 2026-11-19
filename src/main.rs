use file_syncer::{
    config, executor, index, planner, state,
    types::{Command, SyncerError},
    watcher::DirectoryWatcher,
};
use std::time::Duration;

fn main() -> Result<(), SyncerError> {
    let cli_command = config::parse_args()?;

    // Improvement Plan:
    // -> Remove clone as it takes more memory than reasonable
    match cli_command.clone() {
        Command::Watch {
            source: _,
            destination: _,
            config_path,
            verbose: _,
            dry_run: _,
            debounce,
        } => {
            // Config building
            let toml_config = config::load_toml_config(&config_path)?;
            let runtime = config::build_runtime(cli_command, toml_config)?;

            // Resources building
            let db = state::init_db(&runtime.source)?;
            let (_watcher_handle, rx) =
                DirectoryWatcher::spawn(runtime.source.clone(), runtime.ignore_patterns.clone())?;
            // Helpers
            let debounce_duration = Duration::from_millis(debounce);

            if runtime.verbose {
                println!(
                    "Daemon active. Watching {} for changes...",
                    runtime.source.display()
                );
            }

            // Defines the sync cycle as a closure so we can call it on startup AND on events
            let run_sync_cycle = || -> Result<(), SyncerError> {
                if runtime.verbose {
                    println!("Starting reconciliation cycle...");
                }

                let source_map =
                    index::scan_directory_tree(&runtime.source, &runtime.ignore_patterns)?;
                let dest_map =
                    index::scan_directory_tree(&runtime.destination, &runtime.ignore_patterns)?;
                let plan = planner::generate_sync_plan(
                    &source_map,
                    &dest_map,
                    &db,
                    &runtime.conflict_strategy,
                )?;

                if !plan.is_empty() {
                    if runtime.dry_run {
                        println!("DRY RUN: {} pending operations.", plan.len());
                        // Assuming SyncAction derives Debug
                        for action in plan {
                            println!(" - {:?}", action);
                        }
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

                if runtime.verbose {
                    println!("FS mutation detected.");
                }

                // Catch errors locally to prevent the daemon from panicking
                if let Err(e) = run_sync_cycle() {
                    eprintln!("Daemon Error during cycle: {}. Resuming watch...", e);
                }
            }
        }
        Command::StopSpecific {
            source,
            destination,
            config_path,
            verbose,
            dry_run,
            debounce,
        } => {}
        Command::StopAllFrom {
            source,
            config_path,
            verbose,
            dry_run,
            debounce,
        } => {}
        Command::StopAllTo {
            destination,
            config_path,
            verbose,
            dry_run,
            debounce,
        } => {}
    }

    Ok(())
}
