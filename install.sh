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
ExecStart=$INSTALL_DIR/file-syncer watch $SOURCE_DIR $DEST_DIR --config-path $CONFIG_PATH --verbose
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