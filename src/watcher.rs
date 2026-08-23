use std::{path::{Path, PathBuf}, sync::mpsc::Sender};
use notify;
use crate::types::{SyncerError, SyncEvent};

pub fn start_watcher (src_root: &PathBuf, tx: Sender<SyncEvent>, debounce: u64) -> Result<(), SyncerError> {
    todo!("Implement the watcher logic!");
}
