# File Syncer

A high-performance, multi-threaded bidirectional file synchronization engine built in Rust.

## Features

* **Bidirectional Synchronization:** Accurately detects and reconciles changes originating from either the source or destination directory.
* **State-Aware Deletions:** Tracks historical file states to distinguish between new files and deletions.
* **Conflict Resolution:** Handles cryptographic hash collisions seamlessly with configurable strategies.
* **Deferred Interactive Batching:** Executes automated fast-path I/O immediately while queueing interactive decisions for later.
* **Real-time Monitoring:** Reacts instantly to filesystem events using low-level OS hooks.
* **Configurable Debounce:** Fine-tune event batching to smooth out noisy OS file handlers.
* **Dry-Run Support:** Preview path resolution and synchronization steps safely without touching disk.
* **Systemd Integration:** Includes a deployment script to run the engine as a resilient background daemon.

## Built With (Crates & Dependencies)

* **[`sled`](https://crates.io/crates/sled):** Embedded B-Tree database for fast, durable state storage.
* **[`notify`](https://crates.io/crates/notify):** Cross-platform filesystem event monitoring.
* **[`clap`](https://crates.io/crates/clap):** Command-line argument parsing and derive macros.
* **[`tempfile`](https://crates.io/crates/tempfile):** Isolated temporary directory handling for safe testing.
* **[`thiserror`](https://crates.io/crates/thiserror):** Convenient derive macro for error handling.

## Installation

### Option 1: Background Daemon (Recommended for Linux)

Download the latest release binary and configure it to run automatically on boot via a systemd user service.

```bash
curl -LO https://github.com/yourusername/file-syncer/releases/download/v0.2.0/file-syncer-0.2.0-linux-x86_64.tar.gz
tar -xzf file-syncer-0.2.0-linux-x86_64.tar.gz
cd file-syncer-0.2.0
chmod +x install.sh
./install.sh
```

### Option 2: Manual Build

Build the release binary directly with Cargo:

```bash
curl -LO https://github.com/yourusername/file-syncer/archive/refs/tags/v0.2.0.tar.gz
tar -xzf v0.2.0.tar.gz
cd file-syncer-0.2.0
cargo build --release
```

The optimized production executable will be placed in `target/release/file-syncer`.

## Usage

Basic live synchronization using the daemon watcher:

```bash
./target/release/file-syncer watch /path/to/source /path/to/dest --config-path ~/.config/file-syncer/config.toml
```

Run a dry run with verbose execution logging:

```bash
./target/release/file-syncer watch /path/to/source /path/to/dest --verbose --dry-run
```

## CLI Options

| Flag / Option | Description |
|---|---|
| `<SOURCE>` | Absolute or relative path to the monitored source directory. |
| `<DEST>` | Path to the target destination directory. |
| `--config-path <PATH>` | Path to the TOML configuration file. |
| `-v, --verbose` | Enable real-time logging for path calculations and sync actions. |
| `--dry-run` | Process events and calculate destination paths without touching disk. |
| `--debounce <MS>` | Event debounce duration in milliseconds (default: `500`). |
| ... | |

## Installation Script Details (`install.sh`)

The included `install.sh` script automates a production-ready Linux setup:

```bash
#!/bin/bash

# Exit on any error
set -e

echo "Building release binary..."
cargo build --release

# 1. Install the binary to your local path
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/file-syncer "$INSTALL_DIR/"
echo "Binary installed to $INSTALL_DIR/file-syncer"

# 2. Define your sync directories and config (Modify these as needed!)
SOURCE_DIR="$HOME/sync_source"
DEST_DIR="$HOME/sync_dest"
CONFIG_PATH="$HOME/.config/file-syncer/config.toml"

# Ensure directories exist
mkdir -p "$SOURCE_DIR" "$DEST_DIR" "$(dirname "$CONFIG_PATH")"

# 3. Create the systemd user service file
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$SERVICE_DIR/file-syncer.service"

mkdir -p "$SERVICE_DIR"

cat > "$SERVICE_FILE" << EOF
[Unit]
Description=File Synchronizer Daemon
After=network.target

[Service]
Type=simple
# Execute the binary from your local bin
ExecStart=$INSTALL_DIR/file-syncer watch $SOURCE_DIR $DEST_DIR --config-path $CONFIG_PATH
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

echo "Systemd service created at $SERVICE_FILE"

# 4. Reload systemd and enable the service
systemctl --user daemon-reload
systemctl --user enable --now file-syncer.service

echo "Installation complete! The daemon is now running in the background."
```

### How to Use It

1. Make the script executable:

   ```bash
   chmod +x install.sh
   ```

2. Run it:

   ```bash
   ./install.sh
   ```

Because we copied the binary to `~/.local/bin/`, you can also now type `file-syncer` anywhere in your terminal if you ever want to run a manual command or a dry run without using the daemon. (Note: Ensure `~/.local/bin` is in your shell's `$PATH`.)

### Managing Your Service

Now that systemd is managing your engine, you use the standard `systemctl` commands to interact with it. Because it is a `user` service (protecting your personal files), you do not need `sudo`.

* **Check the status:**

  ```bash
  systemctl --user status file-syncer
  ```

* **Stop the synchronizer:**

  ```bash
  systemctl --user stop file-syncer
  ```

* **Start it again:**

  ```bash
  systemctl --user start file-syncer
  ```

* **View your live daemon logs:**

  ```bash
  journalctl --user -u file-syncer -f
  ```

This setup guarantees your daemon will quietly spin up every time you log into your desktop environment, faithfully tracking state and resolving conflicts in the background.

## License

MIT