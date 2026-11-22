use file_syncer::{
    config, executor, index, planner, server, state,
    types::{Command, SyncerError},
    watcher::DirectoryWatcher,
};

fn main() {
    let cli_command = config::parse_args()?;

    todo!("Introduce logic to start server or start client at request");
    todo!(
        "Introduce logic to match a stop request and kill the connection on both clients and the server"
    );
}
