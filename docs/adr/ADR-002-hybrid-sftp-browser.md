# ADR-002: Hybrid SFTP Browser

## Status
Proposed

## Context

sshm-rs currently delegates file transfer to external system commands: `sftp` (launched via `x` key) and `scp` (via a form triggered by `X`). Both shell out and lose integration with the TUI. Users want a richer experience:

1. **Mode B (Auto-split)** -- When connecting to a host via Enter, detect the terminal multiplexer (tmux, WezTerm) and automatically open an SFTP file browser in an adjacent pane. The SFTP browser is `sshm-rs` itself invoked with a special CLI flag.
2. **Mode A (Standalone)** -- An integrated two-panel SFTP browser (local | remote) rendered entirely within ratatui, accessible via a keybinding on any terminal.

The project already uses `ssh2 = "0.9"` which provides `Session::sftp()` for programmatic SFTP access. Passwords are stored via the `keyring` crate. The `PingManager` demonstrates the established pattern of background threads communicating over `mpsc` channels.

## Decision

### 1. Terminal Detection Logic

Introduce a `TerminalEnv` enum and a detection function in a new module `src/terminal_detect.rs`:

```
enum TerminalEnv {
    Tmux,
    WezTerm,
    Unknown,
}
```

Detection order:
1. Check `$TMUX` environment variable (non-empty means tmux).
2. Check `$TERM_PROGRAM == "WezTerm"`.
3. Fallback: `Unknown` -- no auto-split, SSH-only connection.

A `split_pane(env: TerminalEnv, command: &str)` function issues the appropriate shell command:
- **Tmux**: `tmux split-window -h '<command>'`
- **WezTerm**: `wezterm cli split-pane --right -- <command>`

This function is called from the post-TUI action handler in `src/ui/mod.rs` after the TUI exits for an SSH connection, but only when Mode B is enabled.

### 2. CLI Interface for Mode B

Add a new subcommand to the `Commands` enum in `src/cli/mod.rs`:

```
Sftp {
    /// Host to browse via SFTP
    host: String,
    /// SSH config file
    #[arg(short = 'c', long = "config")]
    config_file: Option<String>,
}
```

Invoked as `sshm-rs sftp <host>`. This subcommand:
1. Resolves host details from SSH config (reusing `parse_ssh_config`).
2. Establishes an `ssh2::Session` (see section 5 for auth flow).
3. Calls `session.sftp()` to obtain the SFTP channel.
4. Enters the SFTP Browser TUI (Mode A view), showing only the file browser -- no host list.

The auto-split from Mode B spawns: `sshm-rs sftp <host>` in the adjacent pane.

### 3. Mode A -- SFTP Browser TUI Design

#### 3.1 New ViewMode Variant

Add `SftpBrowser` to the existing `ViewMode` enum. When triggered from the host list (new keybinding `F` on a selected host), the app transitions to `ViewMode::SftpBrowser`.

When launched via `sshm-rs sftp <host>`, the app starts directly in this view mode and exits when the user presses `Esc` or `q`.

#### 3.2 State Structs

New file: `src/ui/sftp_state.rs`

```
struct FileEntry {
    name: String,
    size: u64,
    entry_type: FileType,      // File, Directory, Symlink
    permissions: u32,           // Unix mode bits
    modified: Option<i64>,      // Unix timestamp
}

enum FileType {
    File,
    Directory,
    Symlink,
}

enum ActivePanel {
    Local,
    Remote,
}

struct PanelState {
    path: PathBuf,              // Current directory (absolute)
    entries: Vec<FileEntry>,    // Directory listing
    selected: usize,            // Cursor position
    offset: usize,              // Scroll offset
    loading: bool,              // True while listing in progress
    error: Option<String>,      // Last error message
}

struct SftpBrowserState {
    host_name: String,
    local_panel: PanelState,
    remote_panel: PanelState,
    active_panel: ActivePanel,
    // Channel to send commands to SFTP background thread
    cmd_tx: mpsc::Sender<SftpCommand>,
    // Channel to receive responses
    resp_rx: mpsc::Receiver<SftpResponse>,
    // Transfer progress
    transfer_progress: Option<TransferProgress>,
    // Confirmation dialog
    confirm_dialog: Option<ConfirmDialog>,
}

struct TransferProgress {
    filename: String,
    bytes_transferred: u64,
    bytes_total: u64,
    direction: TransferDirection,  // Upload or Download
}
```

#### 3.3 Background SFTP Thread

New file: `src/connectivity/sftp_worker.rs`

A dedicated thread owns the `ssh2::Sftp` handle (which is not `Send` by default in ssh2). The thread holds both the `Session` and `Sftp` objects and processes commands sequentially from an `mpsc::Receiver<SftpCommand>`:

```
enum SftpCommand {
    ListDir(String),                           // remote path
    Download { remote: String, local: String },
    Upload { local: String, remote: String },
    Mkdir(String),
    Delete(String),                            // file or empty dir
    Rename { from: String, to: String },
    Stat(String),
    Shutdown,
}

enum SftpResponse {
    DirListing { path: String, entries: Vec<FileEntry> },
    TransferProgress { bytes: u64, total: u64 },
    TransferComplete { path: String },
    Error(String),
    Ok,
}
```

The worker thread is spawned when entering the SFTP browser and terminated (via `SftpCommand::Shutdown`) when leaving. This follows the same pattern as `PingManager` but is 1:1 (one thread per SFTP session).

**Why a dedicated thread instead of async**: The project does not use tokio or any async runtime. `ssh2` is synchronous. Adding async would be a significant architectural change for marginal benefit in this use case. A background thread with mpsc channels is consistent with the existing `PingManager` pattern.

#### 3.4 Layout

Two-panel split rendered in ratatui:

```
+------------------------------------------------------+
| Local: /home/user/Documents          | F  sshm-rs    |
|--------------------------------------|---------------|
| [..] (parent)                        | [..] (parent) |
| > my_project/                        |   configs/    |
|   notes.txt           12.4 KB        |   deploy.sh   |
|   report.pdf          2.1 MB         |   logs/       |
|                                      |               |
+------------------------------------------------------+
| [Tab] Switch | [Enter] Open | [F5] Copy | [Esc] Back |
+------------------------------------------------------+
```

- Top bar: breadcrumb path for each panel, host name on remote side.
- File list: icon/indicator for type, name, size (human-readable), modified date.
- Bottom bar: context-sensitive keybinding hints.
- Transfer progress bar overlaid when a transfer is active.

#### 3.5 Key Bindings

| Key       | Action                                           |
|-----------|--------------------------------------------------|
| Tab       | Switch active panel (local <-> remote)           |
| Up/Down   | Navigate entries                                 |
| Enter     | Open directory / trigger edit-in-editor for files |
| Backspace | Go to parent directory                           |
| F5        | Copy selected file(s) between panels             |
| F7        | Create directory in active panel                 |
| F8        | Delete selected file/directory (with confirm)    |
| r         | Rename selected entry                            |
| R         | Refresh both panels                              |
| Esc / q   | Close SFTP browser, return to host list           |

#### 3.6 Local Filesystem Operations

Local panel operations (`std::fs::read_dir`, `std::fs::create_dir`, `std::fs::remove_file`, etc.) run on the main thread since they are fast. Only remote operations go through the SFTP worker thread.

### 4. Edit-in-Editor Flow

When user presses Enter on a remote **file**:

1. Download to a temp directory: `std::env::temp_dir().join("sshm-rs").join(host).join(relative_path)`.
2. Open with `$EDITOR` or `code --wait <file>` (detect VS Code preference via `$VISUAL`, `$EDITOR`, or fallback to `code`).
3. Use `code --wait` semantics: the call blocks until the editor tab is closed.
4. After editor returns, compare file hash (SHA-256) with pre-edit hash. If changed, re-upload via SFTP.

**File-watching (notify crate) is deferred to full scope.** The `--wait` approach is simpler, requires no new dependency, and covers the primary use case. The `notify` crate can be added later for a more seamless experience where the file re-uploads on every save.

### 5. Connection Flow for SFTP

New function in `src/connectivity/mod.rs`:

```
pub fn create_sftp_session(host: &str, config_file: Option<&str>)
    -> Result<(ssh2::Session, ssh2::Sftp)>
```

Authentication cascade:
1. **Password from keyring**: `credentials::get_password(host)` -> `session.userauth_password()`.
2. **SSH agent**: `session.userauth_agent(user)` -- works for hosts using key-based auth with an agent (ssh-agent, pageant, 1Password SSH agent).
3. **Identity file**: If `SshHost.identity` is set, `session.userauth_pubkey_file(user, None, identity_path, None)`.
4. **Interactive keyboard auth**: `session.userauth_keyboard_interactive()` as last resort for hosts requiring challenge-response.

If all methods fail, return an error. The existing `connect_ssh()` function can be refactored to share this auth cascade (currently it only tries password then falls back to system ssh).

The `TcpStream` connection reuses the same pattern as `connect_ssh2_interactive`: resolve host, `TcpStream::connect_timeout`, `Session::new`, `set_tcp_stream`, `handshake`.

### 6. Impact on Existing Code

#### New Files

| File | Purpose |
|------|---------|
| `src/terminal_detect.rs` | Terminal environment detection + split-pane logic |
| `src/connectivity/sftp_worker.rs` | Background SFTP thread, command/response types |
| `src/ui/sftp_state.rs` | `SftpBrowserState`, `FileEntry`, panel state |
| `src/ui/views/sftp_browser.rs` | Ratatui rendering for the two-panel view |
| `src/ui/sftp_event.rs` | Key event handling for SFTP browser mode |

#### Modified Files

| File | Change |
|------|--------|
| `src/cli/mod.rs` | Add `Sftp` subcommand variant + handler |
| `src/ui/app.rs` | Add `SftpBrowser` to `ViewMode`; add `Option<SftpBrowserState>` field to `App` |
| `src/ui/mod.rs` | Handle `ViewMode::SftpBrowser` in draw/event loop; handle auto-split in post-TUI actions |
| `src/ui/event.rs` | Add `F` keybinding to launch SFTP browser from host list |
| `src/ui/views/mod.rs` | Register `sftp_browser` view module |
| `src/ui/views/help.rs` | Add `F` key to help screen |
| `src/connectivity/mod.rs` | Extract auth cascade into `create_sftp_session`; refactor `connect_ssh2_interactive` to reuse it |
| `src/main.rs` | Register `terminal_detect` module |
| `Cargo.toml` | Add `sha2` crate (for edit-then-reupload hash check) |

#### Estimated Complexity

- **Terminal detection + CLI subcommand**: Small (~150 LOC). Low risk.
- **SFTP worker thread**: Medium (~300 LOC). Moderate risk -- ssh2 SFTP error handling, large file transfers.
- **SFTP Browser TUI (rendering + events)**: Large (~600-800 LOC). The bulk of the work. Similar complexity to the existing host list view.
- **Edit-in-editor flow**: Small (~100 LOC). Low risk.
- **Auth cascade refactor**: Small (~100 LOC). Low risk, improves existing code.

**Total estimate**: ~1200-1500 LOC of new/modified code.

### 7. MVP Scope vs Full Scope

#### MVP (Phase 1)

1. `sshm-rs sftp <host>` CLI subcommand with two-panel browser.
2. SFTP worker thread with `ListDir`, `Download`, `Upload`, `Mkdir`, `Delete`.
3. Basic two-panel rendering: navigate, open dirs, copy files between panels.
4. Auth cascade: password from keyring + SSH agent + identity file.
5. `F` keybinding from host list to open SFTP browser inline.
6. Terminal detection (WezTerm, tmux) stored but **not yet wired** to auto-split.

#### Phase 2

1. Auto-split on Enter (Mode B): detect terminal, spawn `sshm-rs sftp <host>` in split pane.
2. Edit-in-editor: download, open with `$EDITOR`/`code --wait`, detect changes, re-upload.
3. Transfer progress bar with percentage and speed.
4. Rename operation.
5. Symlink resolution and display.

#### Phase 3 (Future)

1. File-watching with `notify` crate for live re-upload on save.
2. Multi-file selection and batch transfer.
3. Drag-and-drop (if terminal supports OSC 52 or similar).
4. Bookmarked remote paths per host (persisted in history).
5. Directory diff / sync mode.

## Alternatives Considered

### A. Async runtime (tokio) for SFTP operations
- **Pros**: Natural concurrency model, could enable parallel transfers.
- **Cons**: Adds tokio as a heavy dependency, requires restructuring the entire app (currently fully synchronous), `ssh2` is not async-native (would need `async-ssh2-lite` or `russh` which is a different SSH library). Massive scope increase.
- **Verdict**: Rejected. The synchronous thread + mpsc pattern is proven in this codebase and sufficient for sequential SFTP operations.

### B. Use `russh` instead of `ssh2` for SFTP
- **Pros**: Pure Rust, async-native, actively maintained.
- **Cons**: Different API from `ssh2` (already in use for SSH connections), would require rewriting existing `connect_ssh2_interactive`, two SSH libraries in the same binary is wasteful.
- **Verdict**: Rejected. Stick with `ssh2` for consistency. Can revisit if `ssh2` proves limiting.

### C. Always shell out to system `sftp` command (no integrated browser)
- **Pros**: Zero implementation effort, works everywhere.
- **Cons**: This is what already exists (`x` key). No TUI integration, no dual-panel experience, no edit-in-editor, no progress tracking.
- **Verdict**: Rejected. Does not meet user requirements.

### D. Embed a full file manager (e.g., fork yazi integration)
- **Pros**: Rich file management out of the box.
- **Cons**: Enormous dependency, tight coupling to another TUI app, conflicting event loops with ratatui. Not practical.
- **Verdict**: Rejected.

### E. Auto-split as the only mode (no integrated TUI browser)
- **Pros**: Simpler -- just spawn sftp in a pane, no new TUI views needed.
- **Cons**: Only works in tmux/WezTerm, no fallback for other terminals (plain Windows Terminal, SSH sessions, etc.). Fragile.
- **Verdict**: Rejected as sole approach. Auto-split is Mode B, complementing Mode A.

## Consequences

### Positive
- Fully integrated SFTP experience without leaving sshm-rs.
- Works on any terminal (Mode A), with enhanced experience on tmux/WezTerm (Mode B).
- Reuses existing `ssh2` dependency -- no new SSH library needed.
- Auth cascade improvement benefits both SSH connections and SFTP.
- Phased delivery reduces risk: MVP is useful on its own.

### Negative
- Significant new surface area (~1200-1500 LOC) to maintain.
- `ssh2::Sftp` is not `Send`, requiring the dedicated worker thread to own both Session and Sftp. This means one thread per SFTP session (acceptable for single-host browsing).
- The `code --wait` approach for edit-in-editor blocks the SFTP browser's editor flow until the user closes the file tab. This is a UX limitation mitigated by running the editor launch in a separate thread.
- New `sha2` dependency for file change detection (small, well-maintained, justified).

### Trade-offs
- Chose simplicity (sync threads) over power (async). This means no parallel multi-file transfers in MVP, but avoids the complexity tax of an async runtime.
- Chose `--wait` editor pattern over file-watching. Simpler but less seamless. File-watching is a Phase 3 enhancement.
- Auto-split (Mode B) is Phase 2, not MVP. Users on tmux/WezTerm will not get the split-pane experience immediately but will have the full integrated browser.

## Date
2026-03-05
