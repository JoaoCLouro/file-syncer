use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event};
use crate::types::SyncerError;

/// Handle that manages the background OS watcher thread.
/// Dropping this handle cleans up the underlying OS event subscription.
pub struct DirectoryWatcher {
    _watcher: RecommendedWatcher,
}

impl DirectoryWatcher {
    /// Spawns the native OS watcher on a background thread and returns an event receiver.
    pub fn spawn(watch_path: PathBuf, ignore_patterns: Vec<String>) -> Result<(Self, Receiver<Event>), SyncerError> {
        let (tx, rx) = mpsc::channel();

        // Build the event handler callback to filter events cross-thread
        let event_handler = move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                // Ignore events touching internal database files or user ignore patterns
                let should_skip = event.paths.iter().any(|path| {
                    let path_str = path.to_string_lossy();
                    path_str.contains(".syncer_state") 
                        || ignore_patterns.iter().any(|pattern| path_str.contains(pattern))
                });

                if !should_skip {
                    let _ = tx.send(event);
                }
            }
        };

        // Instantiate platform-recommended watcher
        let mut watcher = RecommendedWatcher::new(event_handler, Config::default())
            .map_err(|e| SyncerError::ValidationError(e.to_string()))?;

        // 3. Register path for recursive watching
        watcher.watch(&watch_path, RecursiveMode::Recursive)
            .map_err(|e| SyncerError::ValidationError(e.to_string()))?;

        Ok((Self { _watcher: watcher }, rx))
    }
}