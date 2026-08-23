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
