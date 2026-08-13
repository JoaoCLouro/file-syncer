# File Syncer — Central Server Architecture

**A flexible, bidirectional file synchronization system** with optional central server management for multi-device scenarios.

Keep your projects synced across any number of locations with intelligent conflict resolution, access control, and real-time or offline support.

---

## Project Overview

File Syncer supports **two deployment modes:**

### Mode 1: Direct Peer-to-Peer Sync (Simpler)
- Sync between two locations directly
- **Use cases:** Dual-boot partitions, desktop ↔ laptop on same network, USB/external drive syncing
- **No server required** — lower complexity, easier setup
- **Limited to:** Two-way sync only

### Mode 2: Central Server-Managed Sync (Recommended for Multi-Device)
- Central hub (Raspberry Pi, mini server, NAS) manages all syncs
- Multiple clients (desktop, laptop, external drives) connect to server
- **Use cases:** Teams, multiple machines offline, centralized access control, audit trails
- **Server benefits:**
  - Single source of truth for file versions
  - Access control (which devices can read/write which files)
  - Conflict resolution at the hub (consistent decisions across all clients)
  - Offline queuing (clients sync changes when reconnected)
  - Audit trail and file history
  - Multiple devices sync through one hub

---

## Architecture: Central Server Mode

### System Diagram

```
┌─────────────────────────────────────────────────────────┐
│           Sync Server (Raspberry Pi / Mini PC)          │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Sync Daemon (Rust)                                │ │
│  │  - HTTP API for clients                            │ │
│  │  - Conflict resolution engine                      │ │
│  │  - Access control & permissions                    │ │
│  │  - File versioning & audit log                     │ │
│  │  - State management (who has what)                 │ │
│  └────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────┐ │
│  │  Authoritative Storage                             │ │
│  │  (Master copy of all files)                        │ │
│  └────────────────────────────────────────────────────┘ │
└──────────────┬────────────────────────────────────────────┘
               │
       ┌───────┼───────┬──────────┐
       │       │       │          │
    ┌──▼──┐ ┌──▼──┐ ┌──▼──┐ ┌─────▼────┐
    │     │ │     │ │     │ │          │
   Desktop Laptop Mobile External USB
   (Client) (Client) (Client) (Client)
```

**Key Components:**

1. **Sync Server (Central Hub)**
   - Runs daemon listening on network
   - Stores authoritative file versions
   - Manages conflicts intelligently
   - Tracks access logs and file history
   - Controls who can access what

2. **Client Applications**
   - Run on each device (desktop, laptop, etc.)
   - Watch local directories for changes
   - Upload changes to server on sync
   - Download changes from server
   - Queue changes if disconnected, sync when online

3. **Network Communication**
   - HTTP REST API (simple, works over any network)
   - Supports WiFi, Ethernet, USB tethering, mobile hotspot
   - Works offline (client queues changes locally)

---

## Core Problem Statements

### Scenario 1: Dual-Boot Systems
When dual-booting (Windows/Arch Linux), developers often:
- Have the same projects on both partitions but can't easily keep them in sync
- Manually copy files after editing, risking version conflicts
- Lose changes when forgetting to sync before rebooting

**File Syncer P2P solves this:** Watch both partitions, auto-sync changes.

### Scenario 2: Multiple Machines with a Central Server
Team or individual with desktop, laptop, and external backup:
- Desktop and laptop work offline, sync when they meet (at home/office)
- External drive serves as portable backup synced from all machines
- Need to know who changed what, when, and resolve conflicts fairly
- Can't rely on cloud storage (privacy, offline requirements)

**File Syncer Server solves this:** Central Pi/server is the hub, all devices sync through it.

**Workflow:**
- **At home:** Desktop and Pi on same WiFi. Desktop syncs to Pi in real-time.
- **At work:** Laptop works offline, queues changes locally.
- **Going home:** Laptop connects to home WiFi, automatically syncs with Pi.
- **Backup:** External drive plugged into Pi syncs all files as backup.
- **Audit:** Server tracks: "desktop changed app.py at 3pm, laptop conflicted at 4pm, resolved to desktop version."

### Scenario 3: Offline Team with Intermittent Connectivity
Remote team members, unreliable internet:
- Work offline on their own laptop, sync to Pi when connection available
- Pi is either portable (brought to sync points) or fixed (when all devices converge)
- Need to resolve conflicts made in parallel when offline
- Need history of who made what changes

**File Syncer Server solves this:** Server queues changes from offline clients, merges intelligently when all connect.

### Scenario 4: Distributed Backup
Want code on multiple devices, with a central source of truth:
- Master copy on Pi (or NAS)
- Automatic backup to external USB when plugged into Pi
- Desktop and laptop both pull from Pi as backup
- Never work directly from Pi (read-only sync to Pi)

**File Syncer Server solves this:** Pi is read-only source for all clients, external USB is automatic backup.

---

## Features Roadmap

### ✅ MVP (Weeks 1-2): P2P One-Way Sync Foundation

**Core Functionality:**
- Watch a source directory for file changes
- One-way sync: propagate changes to destination
- CLI command: `syncer watch <source> <dest>`
- Basic error handling
- Console output showing sync activity

**What You'll Learn:**
- File system APIs
- Error handling with Result/Option
- Structs and pattern matching
- Basic async with tokio

**Deliverable:** Working one-way P2P sync

---

### 🔄 v1.5 (Weeks 3-4): P2P Bidirectional & Conflicts

**New Features:**
- Bidirectional sync (both directions)
- Conflict detection (same file changed both sides)
- Conflict resolution strategies (newer wins, ask user, manual)
- Ignore patterns (`.gitignore`-style)
- Status command showing pending/conflicted files

**What You'll Learn:**
- Two-way async operations
- State tracking and race conditions
- Configuration file parsing
- User interaction in CLI

**Deliverable:** Bidirectional P2P sync with conflict resolution

---

### 🖥️ v2 (Weeks 5-6): Central Server Mode

**New Architecture:**
- **Server Component** (runs on Raspberry Pi or mini server)
  - HTTP API for client connections
  - Stores authoritative file versions
  - Manages conflicts from multiple clients
  - Tracks file history and changes

- **Client Component** (runs on each device)
  - Watches local directories
  - Communicates with server via HTTP
  - Queues changes if offline
  - Auto-syncs when connected

**Features:**
- **Server:**
  - Listen on HTTP port for client connections
  - Authentication (API keys or username/password)
  - File upload/download endpoints
  - Change log endpoint (audit trail)
  - Conflict resolution endpoint
  - Multi-user access control

- **Client:**
  - `syncer client connect <server-url>`
  - `syncer client watch <local-dir>`
  - Watches local directory, reports changes to server
  - Downloads latest from server on sync
  - Queues changes if disconnected
  - Auto-syncs when connection restored

- **Daemon Mode:**
  - Server daemon continuously manages syncs
  - Client daemon continuously uploads/downloads changes
  - Configuration file for server settings (port, storage path, access rules)

**Example Configuration:**

**Server (`server-config.toml`):**
```toml
[server]
host = "0.0.0.0"
port = 9999
storage_path = "/mnt/syncer-storage"
log_path = "/var/log/syncer-server.log"

[auth]
api_key = "secret-server-key-12345"

[[access_rule]]
device = "desktop"
paths = ["/Projects", "/Documents"]
permissions = "read-write"

[[access_rule]]
device = "laptop"
paths = ["/Projects"]
permissions = "read-write"

[[access_rule]]
device = "external-backup"
paths = ["/Projects", "/Documents"]
permissions = "read-only"
```

**Client (`client-config.toml`):**
```toml
[client]
name = "desktop"
server_url = "http://192.168.1.100:9999"
api_key = "secret-server-key-12345"
storage_path = "/home/user/Projects"
sync_mode = "realtime"

[conflict_resolution]
strategy = "newer"  # or "ask", "manual"
```

**What You'll Learn:**
- HTTP server implementation (tokio-based)
- RESTful API design
- Client-server communication patterns
- Authentication and authorization
- File versioning and history tracking
- Offline-first architecture (queuing changes)
- Multi-client state management

**Deliverable:** Working central server + client architecture

---

### 📋 Future Enhancements (Post-v2)

- **WebSocket support** — Real-time sync updates instead of polling
- **Delta sync** — Only transfer file changes (diffs), not full files
- **Compression** — Compress files during transfer (bandwidth optimization)
- **Encryption** — End-to-end encryption for files at rest and in transit
- **Web UI** — Browser dashboard on server showing sync status, conflicts, file history
- **Undo/Rollback** — Recover previous versions of files from server history
- **Mobile clients** — Android/iOS apps syncing to central server
- **Multi-device conflict resolution** — When 3+ devices change same file offline
- **Selective sync** — Client can choose which folders to sync
- **Bandwidth throttling** — Limit upload/download speed
- **File locking** — Prevent multiple clients from editing same file simultaneously

---

## Technical Architecture

### P2P Mode (v1-v1.5)
```
Device A ←→ File System Watcher
   ↓
   Watch and detect changes
   ↓
Device B ←→ File System Operations
```

### Server Mode (v2+)
```
┌──────────────────────────┐
│   Server (Sync Hub)      │
│  - HTTP API              │
│  - File Storage          │
│  - Conflict Resolution   │
│  - Audit Log             │
└────────────┬─────────────┘
             │
      ┌──────┼──────┬──────┐
      ↓      ↓      ↓      ↓
   Desktop Laptop Mobile External
   (Client) (Client) (Client) (Client)
     ↓        ↓       ↓        ↓
   Watch    Watch   Watch    Watch
   Local    Local   Local    Local
   Dir      Dir     Dir      Dir
```

### Data Flow: Client to Server
```
1. Client watches local directory
2. File changes detected
3. Client sends to server:
   - File path
   - Content hash
   - Timestamp
   - Device ID
4. Server receives, checks:
   - Authorization (can this device write?)
   - Conflicts (was file also changed elsewhere?)
   - Version (is this newer than current?)
5. If conflict, server decides:
   - Newer wins? Apply change
   - Manual? Mark for resolution, notify all clients
   - Ask? Prompt user
6. Server updates authoritative copy
7. Server notifies all other clients of change
8. Other clients download updated files
```

---

## Technical Requirements

### Language & Tooling
- **Language:** Rust (Chapters 1-10 complete, learning 11+ as needed)
- **Build:** Cargo
- **Edition:** 2021

### Key Crates

**P2P Mode (v1-v1.5):**
- `notify` — File system watching
- `tokio` — Async runtime
- `clap` — CLI argument parsing
- `serde` & `toml` — Configuration
- `tempfile` — Testing

**Server Mode (v2+):**
- Above +
- `axum` or `actix-web` — HTTP server framework
- `tokio` — Async for server
- `serde_json` — JSON serialization
- `uuid` — Unique device/file IDs
- `chrono` — Timestamps
- `sqlx` or `rusqlite` — Database for audit log/state
- `jsonwebtoken` — API authentication

### Platform Support
- **Primary:** Linux (Arch, Raspberry Pi OS)
- **Secondary:** Windows
- **Server:** Any Linux (optimized for Pi/low-power hardware)
- **Clients:** Desktop, laptop, mobile (future)

---

## Implementation Plan

### Weeks 1-2: MVP — P2P One-Way Sync

**Server/Pi Note:** Not needed yet. Just local sync.

Tasks:
1. Set up Cargo project
2. Implement file system watcher using `notify`
3. One-way sync logic (copy new/modified/deleted files)
4. Basic CLI: `syncer watch <source> <dest>`
5. Tests with `tempfile`

**Deliverable:** Working one-way P2P sync

---

### Weeks 3-4: v1.5 — P2P Bidirectional & Conflicts

**Server/Pi Note:** Still not needed. Pure P2P.

Tasks:
1. Bidirectional sync (both directions)
2. Conflict detection (same file changed both sides)
3. Conflict resolution strategies
4. Ignore patterns (`.gitignore`)
5. `syncer status` command
6. Configuration file support

**Deliverable:** Bidirectional P2P sync with conflicts

---

### Weeks 5-6: v2 — Central Server Mode

**Now the Raspberry Pi/mini server comes in.**

Tasks:

**Server Component:**
1. Set up HTTP server with `axum`
2. Create REST API endpoints:
   - `POST /api/sync/upload` — Client uploads file changes
   - `GET /api/sync/download/<file>` — Client downloads files
   - `GET /api/sync/changes-since/<timestamp>` — Client gets what's new
   - `POST /api/sync/resolve-conflict` — Manual conflict resolution
   - `GET /api/history/<file>` — File version history
3. Implement conflict resolution logic
4. Add authentication (API keys)
5. Persist state (SQLite database)
6. Implement audit log (track who changed what when)

**Client Component:**
1. Modify watcher to communicate with server
2. Implement HTTP client code
3. Queue changes locally if offline
4. Upload queued changes when online
5. Download server changes on sync
6. Handle server-side conflicts gracefully

**Daemon Mode:**
1. Server daemon: `syncer server start <config-file>`
2. Client daemon: `syncer client daemon <config-file>`
3. Both daemonize and run in background

**Deliverable:** Full server + client architecture

---

## Example Workflows

### P2P Mode: Dual-Boot
```bash
# Terminal on Windows (or from Arch):
$ syncer watch C:\\Users\\João\\Projects /home/joão/Projects
Watching directories...
> [Windows] main.rs created
> Synced: main.rs (Windows → Arch)
> [Arch] config.toml modified
> Synced: config.toml (Arch → Windows)
```

### Server Mode: Desktop + Laptop + Pi

**Setup Phase (first time):**
```bash
# On Raspberry Pi:
$ syncer server init
? Server storage path: /mnt/syncer-storage
? Server port: 9999
Config saved to /etc/syncer-server.toml

$ syncer server start
Syncer server started on 192.168.1.100:9999
Listening for client connections...

# On Desktop:
$ syncer client init
? Server URL: http://192.168.1.100:9999
? Device name: desktop
? Local sync path: /home/user/Projects
Config saved to ~/.config/syncer/client.toml

$ syncer client daemon
Client daemon started, syncing to http://192.168.1.100:9999
Watching: /home/user/Projects

# On Laptop (same setup):
$ syncer client init
? Server URL: http://192.168.1.100:9999
? Device name: laptop
? Local sync path: /home/user/Projects
...
```

**Daily Usage:**
```bash
# At home, both connected to Pi
[Desktop] File changed: app.py
→ Desktop uploads to Pi
→ Pi notifies Laptop
→ Laptop downloads app.py

# Laptop goes to work (offline)
[Laptop] File changed: main.rs (offline, queued)

# Laptop comes home, connects to WiFi
[Laptop] Connected to server
→ Laptop uploads queued changes
→ Pi checks for conflicts
→ Pi notifies Desktop
→ Desktop downloads changes

# Both changed same file while offline
[Desktop] config.json changed 2pm
[Laptop] config.json changed 3pm
→ Pi detects conflict
→ Pi shows: "desktop (2pm) vs laptop (3pm)"
→ Resolution: "Newer wins" → Use laptop version
→ Both devices updated with laptop version
```

### Server Mode: Offline Team
```bash
# Team member 1 (Laptop A, goes offline)
$ syncer client daemon
[Offline] Working on projects...
> [Local] app.py modified
> [Local] new_feature.rs created
Changes queued locally (no connection)

# Team member 2 (Laptop B, at office with Pi)
$ syncer client daemon
> [Server] Downloaded: data.json (from team member 3)
> [Local] app.py modified
> Synced: app.py (conflict detected)
Conflict: app.py changed both here and on Pi (by team member 1 laptop A)
Resolution: Newer wins → Use my version (I changed it last)

# Team member 1 returns home, connects to Pi
[Laptop A] Connected, syncing...
> Conflict detected in app.py
> Your version (3pm) vs Server version (4pm, from team member 2)
> Resolution strategy: Newer → Using server version
> Downloaded: app.py (from team member 2)
```

---

## Success Criteria

**MVP (Week 2):**
- One-way sync works reliably
- CLI is functional
- Basic tests pass
- GitHub repo has clear documentation

**v1.5 (Week 4):**
- Bidirectional sync works
- Conflicts detected and resolved
- Ignore patterns work
- Status command is useful

**v2 (Week 6):**
- Server starts and listens for clients
- Client connects and syncs files
- Conflicts from multiple clients are resolved
- Audit log tracks changes
- Daemon mode is stable and reliable
- Ready for personal multi-device use or small team

---

## Why This Matters

✅ **Solves real problems:** Dual-boot, multiple machines, offline teams  
✅ **Teaches multiple Rust domains:** Async, file I/O, HTTP, databases, CLI  
✅ **Portfolio-worthy:** Shows systems thinking and architecture depth  
✅ **Scoped well:** MVP → v2 is a natural progression  
✅ **Grows with needs:** P2P for simple cases, server for complex scenarios  
✅ **Self-hosted alternative:** No cloud dependency, full control  

---

## Getting Started

1. Read this entire README to understand the vision
2. Start with **Weeks 1-2 (MVP)** — P2P one-way sync, no server
3. Build incrementally — each phase delivers value on its own
4. The server component (v2) is optional — P2P mode is complete and useful
5. Track progress week-by-week against the implementation plan

---

**Start simple, scale as needed. Begin with the MVP.**
