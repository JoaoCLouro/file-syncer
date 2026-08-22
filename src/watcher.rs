use std::{path::Path, sync::mpsc::Sender};
use notify;
use crate::types::{SyncerError, SyncEvent};

pub fn start_watcher (source: &Path, tx: Sender<SyncEvent>) -> Result<impl notify::Watcher, SyncerError> {
    todo!("Implement the watcher logic!");
}
