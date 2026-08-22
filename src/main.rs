use std::{thread, time::Duration, sync::mpsc};

use clap::Command;

use file_syncer::{cli::parse_args, types::{Command, SyncError}, watcher::start_watcher};

fn main () -> Result<(), SyncerError> {
    let config = parse_args()?;
    // Channel to communicate with the watcher thread
    let (tx, rx) = mpsc::channel();
    match config {
        Command::Watch {source, destination, verbose, debounce} => {
            // watcher runs on a different thread
            let handle = thread::spawn(move || {
                todo!("Call start watcher. Pattern match watcher result and call a sync action.");
                let notice = start_watcher(source, tx);

                // Sleeps through a debouncing period to avoid race conditions or deadlocks
                thread::sleep(Duration::from_millis(debounce));
            });

            handle.join().unwrap();
        }

        // No other command for now.
        // Match other commands as added
    }

    Ok(())
}