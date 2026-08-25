# File Syncer

A fast, lightweight, event-driven file synchronizer built in Rust. It monitors a source directory in real time using OS filesystem notifications and mirrors changes (`Created`, `Modified`, `Deleted`) to a destination directory.

## Features

* **Real-time Monitoring:** Reacts instantly to filesystem events using low-level OS watcher APIs.
* **Recursive Handling:** Automatically mirrors nested directory creation and handles folder cleanup on deletion.
* **Dry-Run Support:** Preview path resolution and synchronization steps safely without writing to disk.
* **Configurable Debounce:** Fine-tune event batching to smooth out noisy OS file handles.

## Built With (Crates & Dependencies)

* **[`clap`](https://crates.io/crates/clap):** Command-line argument parsing using derive macros for declarative CLI structure.
* **[`ctrlc`](https://crates.io/crates/ctrlc):** Cross-platform handling of `Ctrl+C` and `SIGINT` signals for graceful watcher shutdown.
* **[`notify`](https://crates.io/crates/notify):** Cross-platform filesystem event monitoring (uses `inotify` under Linux).
* **[`tempfile`](https://crates.io/crates/tempfile):** Isolated temporary directory generation for unit tests and end-to-end integration testing.
* **[`thiserror`](https://crates.io/crates/thiserror):** Convenient derive macro for implementing custom, structured error types.

## Installation

Build the release binary directly with Cargo:

```bash
git clone [https://github.com/yourusername/file-syncer.git](https://github.com/yourusername/file-syncer.git)
cd file-syncer
cargo build --release
```

The optimized production executable will be placed in `target/release/file-syncer`.

## Usage

Basic live synchronization:

```bash
./target/release/file-syncer --source /path/to/source --dest /path/to/dest
```

Run a dry run with verbose execution logging:

```bash
./target/release/file-syncer --source /path/to/source --dest /path/to/dest --verbose --dry-run
```

## CLI Options

Flag/ Option | Description |
`-s, --source <PATH>` | Absolute or relative path to the monitored source directory. |
`-d, --dest <PATH>` | Path to the target destination directory. |
`-v, --verbose` | Enable real-time logging for path calculations and sync actions. |
`--dry-run` | Process events and calculate destination paths without touching disk.  
`--debounce <MS>` | Event debounce duration in milliseconds (default: `500`). |

## License

MIT
