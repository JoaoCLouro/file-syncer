use std::sync::mpsc;

use file_syncer::{cli::parse_args, sync::process_event, types::{Command, SyncerError, SyncEvent}, watcher::start_watcher};

fn main () -> Result<(), SyncerError> {
    let config = parse_args()?;
    match config {
        Command::Watch {source, destination, verbose, debounce, dry_run} => {
            if !source.exists() {return Err(SyncerError::ValidationError(format!("Source `{}`is not valid", source.display())));}
            
            // Channel to communicate with the watcher thread
            let (tx, rx) = mpsc::channel();
            
            // Sets a ctl-c handler to trigger a graceful exit
            let tx_clone = tx.clone();
            ctrlc::set_handler(move || {
                println!("\nReceived interrupt signal, stopping...");
                let _ = tx_clone.send(SyncEvent::Stop);
            }).expect("Error setting Ctrl-C handler");
            
            // watcher runs on a different thread
            if let Err(e) = start_watcher(&source, tx, debounce) {
                    eprintln!("Fatal error: {e}");
                    std::process::exit(1);
            }
            // Receiving loop
            loop {
                let event = rx.recv().unwrap();
                if let Err(e) = process_event(&event, &source, &destination, verbose, dry_run) {
                    match e {
                        SyncerError::Stop(_) => break,
                        _                     => {
                            eprintln!("Fatal error: {e}");
                            std::process::exit(2);
                        }
                    }
                }
            }
        }

        // No other command for now.
        // Match other commands as added
    }

    Ok(())
}