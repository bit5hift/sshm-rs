# ADR-001: Integrated SFTP File Browser

## Status
Proposed

## Context

sshm-rs is a TUI SSH connection manager (ratatui 0.29 + crossterm 0.28, ~6200 lines of Rust). Currently, file transfer is handled via a simple SCP overlay that shells out to `scp`/`sftp` system commands -- the user types paths manually with no browsing capability. The user wants a split-panel file browser (local | remote) integrated into the TUI, similar to MobaXterm's SFTP browser, allowing keyboard-driven navigation and upload/download without leaving the ratatui interface.

---

## Research Findings

### 1. Rust SFTP Crates

| Crate | Version | Async | Backend | SFTP API | Windows | Maturity |
|---|---|---|---|---|---|---|
| **ssh2** (already a dep) | 0.9 | Sync (blocking) | libssh2 (C) | `Session::sftp()` -> `Sftp` struct with `readdir`, `open`, `create`, `stat`, `rename`, `mkdir`, `rmdir`, `symlink`, `unlink` | Yes (vendored libssh2) | Mature, stable, maintained by alexcrichton |
| **russh** + **russh-sftp** | russh 0.46+ / russh-sftp 2.1.1 | Async (tokio) | Pure Rust | High-level API mirroring `std::fs`: `open_dir`, `read_dir`, `open`, `create`, `stat`, `rename`, `remove_file`, `remove_dir` | Yes (pure Rust) | Actively maintained (Eugeny/Tabby author), good API |
| **openssh-sftp-client** | 0.15+ | Async (tokio) | Pure Rust protocol, requires running `ssh` subprocess | Full SFTP v3 with extensions (fsync, hardlink, posix-rename, copy-data) | Partial (needs OpenSSH binary) | Good but depends on external `ssh` process |
| **remotefs-ssh** | 0.5+ | Sync | libssh2 or libssh | `RemoteFs` trait: `list_dir`, `stat`, `create_file`, `remove_file`, `send_file`, `recv_file` | Yes | Good abstraction layer, used by termscp |

**Key finding**: The `ssh2` crate (v0.9, already in `Cargo.toml`) has full SFTP support via `session.sftp()`. This means zero new SSH dependencies are needed for an MVP. The `Sftp` struct provides: `readdir(path)` returning `Vec<(PathBuf, FileStat)>`, `open(path, flags)`, `create(path)`, `stat(path)`, `mkdir(path, mode)`, `rmdir(path)`, `unlink(path)`, `rename(src, dst)`, and `symlink(target, path)`. File handles implement `Read + Write`.

### 2. TUI File Browser Patterns

| Project/Crate | Approach | Notes |
|---|---|---|
| **ratatui-explorer** | Widget crate wrapping `std::fs` | Lightweight, customizable theme, handles input via `handle()` method. Local FS only. |
| **tui-file-explorer** | Self-contained widget | Directory nav, search, sort (name/size/ext), hidden files toggle. Local FS only. |
| **termscp** | Full app (tui-realm framework) | Split panel (local/remote), supports SCP/SFTP/FTP/S3. Uses `remotefs` trait for protocol abstraction. ~30k+ lines. |
| **yazi** | Async Rust file manager (tokio) | Extremely feature-rich, plugin system. Not embeddable as a library. |

**Typical ratatui file browser pattern**:
- State struct: `current_dir: PathBuf`, `entries: Vec<DirEntry>`, `selected: usize`, `offset: usize`
- Render: `List` or `Table` widget with icons, size, permissions columns
- Input: Up/Down to navigate, Enter to descend/open, Backspace to go up, `/` to search
- This pattern applies equally to local and remote -- only the I/O backend differs

### 3. Architecture Options

#### Option A: Embedded in sshm-rs TUI (new ViewMode with split layout)
- **How**: New `ViewMode::SftpBrowser` with a `SftpBrowserState` struct. Left panel reads local FS via `std::fs`, right panel reads remote FS via `ssh2::Sftp`. Split layout using `Layout::horizontal()`.
- **Pros**: Seamless UX, no external dependencies, reuses existing `ssh2` crate, full control over keybindings and theme, works on Windows without sshfs
- **Cons**: Significant new code (~1500-2500 lines estimated), must handle SSH auth (already solved in `connect_ssh2_interactive`), blocking I/O on the SSH side needs threading or careful non-blocking handling
- **Auth reuse**: The project already has `Session` creation + password auth in `connectivity/mod.rs` (lines 214-278) and keyring integration

#### Option B: Launch external tool (yazi, ranger) with SFTP mount (sshfs)
- **How**: Mount remote FS via `sshfs`, then launch yazi with a split view
- **Pros**: Minimal code (~50 lines), leverages battle-tested tools
- **Cons**: Requires sshfs (not available on Windows natively), yazi has no built-in split local/remote view, user leaves the sshm-rs TUI, poor Windows support, extra system dependency

#### Option C: Separate binary/module that sshm-rs spawns
- **How**: Build a standalone `sshm-sftp` binary that sshm-rs invokes, similar to how it currently shells out to `ssh`/`scp`
- **Pros**: Clean separation of concerns, can iterate independently
- **Cons**: Still need to build the full TUI file browser, loses state continuity with sshm-rs, duplicates SSH config parsing, extra binary to distribute

#### Option D: Embed termscp or remotefs as a library dependency
- **How**: Add `remotefs-ssh` for the SFTP backend, build custom TUI on top
- **Pros**: Protocol abstraction (could support FTP/S3 later), battle-tested file operations
- **Cons**: Adds new dependency (remotefs + its transitive deps), still need to build all the TUI code, remotefs uses libssh2 anyway (same as ssh2 crate)

### 4. Complexity Assessment

**MVP scope** (Option A):
- `SftpBrowserState`: ~200 lines (state management, directory caching)
- `sftp_browser.rs` view: ~400 lines (split layout rendering, file list with icons/sizes)
- `sftp_browser` event handler: ~300 lines (navigation, upload/download, delete, mkdir)
- SFTP session management: ~150 lines (connect, auth reuse, session lifecycle)
- File transfer logic: ~200 lines (upload/download with progress callback)
- Integration glue (ViewMode, keybinding to enter browser): ~50 lines
- **Total estimated: ~1300-1800 lines for MVP**

**Hard problems**:
1. **Blocking I/O**: `ssh2` is synchronous. Directory listings and file transfers block the thread. Must run SFTP ops in a background thread and communicate via channels (same pattern as `PingManager`).
2. **Large directories**: Listing a directory with 10k+ entries. Solution: lazy loading, virtual scrolling (ratatui handles this well).
3. **Progress tracking for transfers**: Need a progress bar for large files. Requires chunked read/write with byte counting.
4. **Symlink resolution**: `readdir` returns symlinks; need to decide whether to follow them.
5. **Permission errors**: Remote directories may be unreadable. Graceful error display needed.
6. **Session keepalive**: Long-idle SFTP sessions may timeout. Need keepalive or reconnect logic.
7. **Concurrent operations**: User might want to browse while a transfer runs. Requires async/threaded transfer.

**NOT hard** (already solved in codebase):
- SSH authentication (password via keyring, key-based via system ssh)
- TUI overlay/popup patterns (many examples in current views)
- Keyboard navigation patterns (List view already implements this)
- Split layouts (sidebar already uses `Layout::horizontal()`)

### 5. Effort Estimation

| Scope | Effort | Timeline |
|---|---|---|
| MVP: browse + download/upload single files | **Medium** | 3-5 days |
| + Progress bars, mkdir, delete, rename | Medium-Large | +2-3 days |
| + Drag-select, batch transfer, queue | **Large** | +3-5 days |
| + Async transfers, resume, diff view | **Huge** | +5-10 days |

---

## Decision

**Option A: Embedded in sshm-rs TUI** using the existing `ssh2` crate's SFTP support.

Rationale:
1. `ssh2` 0.9 is already a dependency and provides complete SFTP operations -- zero new crate additions for the MVP.
2. The codebase already has SSH session creation with password auth and the `PingManager` pattern for background threading, both directly reusable.
3. The existing `ViewMode` enum and overlay pattern make adding a new full-screen view straightforward.
4. Windows support is guaranteed (vendored libssh2, no sshfs needed).
5. Keeping everything in-process means the user never leaves the TUI -- the core UX goal.

## Alternatives Considered

- **Option B (sshfs + yazi)**: Rejected. No Windows support, requires extra system tools, user leaves the TUI.
- **Option C (separate binary)**: Rejected. Same implementation effort as Option A but worse UX (process boundary, no shared state).
- **Option D (remotefs-ssh)**: Rejected. Adds a dependency for the same libssh2 backend we already have. The abstraction layer is useful for multi-protocol support, but sshm-rs only needs SFTP.

## Consequences

**Positive**:
- Feature parity with MobaXterm's SFTP browser within a terminal
- No new runtime dependencies (ssh2 already present)
- Consistent with the existing TUI architecture
- Windows-native without sshfs

**Negative**:
- ~1500+ lines of new code to maintain
- Blocking ssh2 I/O requires careful threading (background thread for all SFTP ops, channel-based communication with TUI thread)
- The `App` struct will grow further (already large at 60+ fields) -- consider extracting `SftpBrowserState` as a separate struct to limit bloat
- Key-based auth via ssh2 is more complex than password auth (need to find and parse identity files); MVP should support password-authed hosts first, with key-based auth as a fast follow

**Migration path**:
- If async becomes necessary later (e.g., concurrent transfers), migrating from `ssh2` to `russh` + `russh-sftp` is feasible since the SFTP API surface is similar. The TUI code would not change.

## Date
2026-03-05
