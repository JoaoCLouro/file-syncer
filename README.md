# File Syncer MVP 
**One-way file synchronization engine in Rust. Watch a source directory and automatically sync changes to a destination.**

---

## What It Does

```
Source Directory          Destination Directory
    (watched)                  (synchronized)
    
File created     → Copied
File modified    → Overwritten
File deleted     → Removed

Changes flow: Source → Destination only (one-way)
```

---

## Architecture Overview

**Two-threaded design:**

```
┌──────────────────────────────┐
│    Main Thread (Sync)        │
│  - Parse CLI arguments       │
│  - Listen on mpsc channel    │
│  - Process sync events       │
│  - Execute I/O operations    │
└──────────────────────────────┘
         ↑ (mpsc channel)
         │
┌──────────────────────────────┐
│  Watcher Thread (notify)     │
│  - Watch source directory    │
│  - Translate OS events       │
│  - Send via mpsc channel     │
└──────────────────────────────┘
```

---

## Installation & Build

### Prerequisites
- **Rust 1.70+** ([Install](https://rustup.rs/))
- **Cargo** (comes with Rust)

### Build
```bash
git clone https://github.com/JoaoCLouro/file-syncer.git
cd file-syncer
cargo build --release
```

Binary: `target/release/file-syncer`

---

## Usage

### Basic Command
```bash
syncer watch <source> <destination>
```

### Examples
```bash
# Dual-boot sync
syncer watch /mnt/windows/Projects /home/user/Projects

# USB backup
syncer watch ~/Documents /media/usb-backup/Documents

# Test directories
syncer watch ./source ./destination
```

### Output
```
Watching: /home/user/Projects → /media/backup/Projects
Connected to file system watcher

[NEW] main.rs
Synced: main.rs

[MODIFIED] config.toml
Synced: config.toml

[DELETED] old-file.rs
Removed: old-file.rs

Watching... (Press Ctrl+C to stop)
```

---

## Features (MVP)

✅ **Real-time watching** — Detects file creates, modifies, deletes  
✅ **One-way sync** — Source → Destination only  
✅ **Cross-platform** — Linux, Windows, macOS  
✅ **Error handling** — Continues on errors (permissions, locks, etc.)  
✅ **Subdirectories** — Syncs entire directory trees  
✅ **Clean CLI** — Simple, intuitive command  
✅ **Tested** — Unit + integration tests included  

---

## Project Structure

```
file-syncer/
├── Cargo.toml
├── README.md (this file)
├── src/
│   ├── main.rs              # Entry point, thread setup
│   ├── lib.rs               # Public module exports
│   ├── types.rs             # Config, SyncEvent, SyncerError
│   ├── cli.rs               # Argument parsing (clap)
│   ├── watcher.rs           # Filesystem watching (notify)
│   └── sync.rs              # Sync engine & I/O operations
├── tests/
│   └── integration_tests.rs  # End-to-end tests (tempfile)
└── docs/
    └── Architecture_and_roadmap.md
```

---

## Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test
```bash
cargo test test_copy_file
```

### Run with Output
```bash
cargo test -- --nocapture
```

### Test Categories

**Unit Tests (sync.rs):**
- Path calculations
- File operations in isolation

**Integration Tests (tests/):**
- Full workflow with temp directories
- Create → Sync → Verify
- Modify → Sync → Verify
- Delete → Sync → Verify

---

## Troubleshooting

### "Permission denied" error
```
Error: Cannot sync file.txt: Permission denied
```
**Fix:** Ensure read/write access to both directories.
```bash
chmod u+rwx /path/to/source /path/to/destination
```

### "File is locked" error
```
Error syncing document.docx: File is locked
```
**Why:** File is open in another application.  
**Fix:** Close the file; syncer will retry automatically.

### "Path not found"
```
Error: Source path does not exist
```
**Fix:** Double-check paths exist:
```bash
ls -la /path/to/source
ls -la /path/to/destination
```

---

## Performance & Limitations

### Performance
- **Throughput:** Sequential (one file at a time)
- **Scalability:** Handles thousands of files fine

### Limitations (MVP)
- **One-way only** — Must run in reverse for bidirectional
- **No conflict resolution** — Later sync wins
- **No ignore patterns** — All files synced (including `.git`)
- **No daemon mode** — Requires terminal to stay open

### What's Safe
✅ Code repositories  
✅ Documents  
✅ Media files  
✅ Backup to USB/external drive  

❌ **Not recommended** — Active bidirectional editing without conflict handling  
❌ **Not recommended** — Mission-critical data without additional backup  

---

## Dependencies

| Crate | Purpose | Version |
|---|---|---|
| `notify` | File system watching | ^5.0 |
| `clap` | CLI argument parsing | ^4.0 (with derive) |
| `tempfile` | Testing (temp directories) | ^3.0 |
| `ctrlc` | Ctrl-C handler | ^3.0 | 

---

## What's Next (Roadmap)

**v1.5** (future):
- Bidirectional sync
- Conflict detection & resolution
- Ignore patterns (`.gitignore`-style)
- `syncer status` command

**v2** (future):
- Daemon mode (background syncing)
- Configuration files
- Central server architecture
- Multi-device syncing

---

## Development Workflow

### Build & Test Locally
```bash
# Debug build (fast compile, slow runtime)
cargo build
cargo test

# Release build (slow compile, fast runtime)
cargo build --release
./target/release/file-syncer watch ./test-src ./test-dest
```

### Code Quality
```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run tests
cargo test
```

---

## Contributing

This is a learning project. Contributions and suggestions welcome!

**Development guidelines:**
1. Code follows Rust conventions (cargo fmt, cargo clippy pass)
2. All tests pass (cargo test)
3. No `unwrap()` — use `?` operator
4. Error types are clear and actionable
5. Modules are focused and testable

---

## License

MIT License — Use freely, modify, distribute.

---

## Quick Start (5 Minutes)

```bash
# Clone and build
git clone https://github.com/JoaoCLouro/file-syncer.git
cd file-syncer
cargo build --release

# Create test directories
mkdir -p /tmp/sync-test/{source,destination}

# Run syncer
./target/release/file-syncer watch /tmp/sync-test/source /tmp/sync-test/destination

# In another terminal, test it
touch /tmp/sync-test/source/hello.txt
ls /tmp/sync-test/destination/  # Should see hello.txt

# Modify and delete
echo "hello" > /tmp/sync-test/source/hello.txt
rm /tmp/sync-test/source/hello.txt

# Watch syncer keep them in sync
```

---

João Carrilho Louro [My Github](https://github.com/JoaoCLouro)