# ADR-001: Host Groups / Sections

## Status
Proposed

## Context

sshm-rs currently displays SSH hosts as a flat table sorted by name or last-used, with favorites pinned to top. Tags exist as metadata parsed from `# Tags:` comments and can filter via a sidebar, but there is no visual grouping or hierarchy. Users with dozens or hundreds of hosts need a way to organize them into logical sections (e.g., "Production", "Staging", "Home Lab") that are visually distinct and collapsible, similar to MobaXterm session folders or VS Code explorer sections.

Key constraints:
- The primary data source is `~/.ssh/config`, which is a standard format used by OpenSSH.
- sshm-rs already stores sidecar data in `~/.config/sshm-rs/` (favorites.json, history.json, snippets.json).
- The table rendering uses ratatui `Table` with `Row` items and manual offset-based scrolling.
- A host can already belong to multiple tags but only one "selected" position in the sorted list.

## Decision

### 1. Data Model: Separate JSON file (`groups.json`)

Store group assignments in `~/.config/sshm-rs/groups.json`, NOT in SSH config comments.

```json
{
  "groups": [
    { "name": "Production", "order": 0, "collapsed": false },
    { "name": "Staging", "order": 1, "collapsed": true },
    { "name": "Home Lab", "order": 2, "collapsed": false }
  ],
  "assignments": {
    "web-prod-01": "Production",
    "web-prod-02": "Production",
    "api-staging": "Staging",
    "nas": "Home Lab"
  }
}
```

Hosts not present in `assignments` belong to a virtual "Ungrouped" section shown last.

### 2. UI Design: Section Header Rows in the Table

Render groups as **non-selectable header rows** interspersed with host rows in the existing `Table` widget. Each header row spans the full width, styled distinctly (bold, dimmed background, with a collapse indicator).

Visual layout:

```
  St  Name             User    Hostname         Port  Tags
  --- ---------------- ------- ---------------- ----- ----------
  v Production (3)
  *   web-prod-01      deploy  10.0.1.1         22    #web
  *   web-prod-02      deploy  10.0.1.2         22    #web
  *   db-prod          dba     10.0.1.10        5432  #db
  > Staging (2)                                         [collapsed]
  v Home Lab (1)
  *   nas              admin   192.168.1.50     22    #storage
  v Ungrouped (4)
  *   dev-vm           user    localhost         2222
  ...
```

- `v` = expanded, `>` = collapsed
- Header rows show group name + host count
- Collapsed groups hide their children, only the header line is visible
- Cursor navigation (Up/Down) skips header rows automatically
- Header rows are visually distinct: bold text, primary color, no status indicator

### 3. Interaction Design

| Action | Key | Behavior |
|---|---|---|
| Toggle collapse | `Enter` on header row, or `Space` | Expand/collapse the group under cursor |
| Create group | `G` (shift+g) | Opens a small input popup for group name |
| Rename group | `R` on a header row | Opens rename popup pre-filled with current name |
| Delete group | `D` on a header row | Confirmation popup; hosts move to Ungrouped |
| Assign host to group | `g` on a host row | Opens a picker listing all groups + "Ungrouped" |
| Move group up/down | `Ctrl+Up/Down` on header row | Reorder groups |
| Collapse all | `zc` or `Ctrl+[` | Collapse every group |
| Expand all | `ze` or `Ctrl+]` | Expand every group |

**Interaction with existing features:**

- **Search/filter**: When a search query or tag filter is active, groups are flattened -- all matching hosts are shown in a flat list without section headers. This avoids empty sections and confusion.
- **Sort**: Sorting applies within each group. Favorites are still pinned to the top within their respective group.
- **Favorites**: A favorite host stays in its group but is sorted to the top of that group.
- **Multi-select**: Only host rows can be selected, not headers.
- **Tag sidebar**: Tag filtering takes priority -- when active, groups are hidden and results are flat.

### 4. Internal Architecture

Introduce a `GroupsManager` struct (new file `src/groups.rs`), following the same pattern as `FavoritesManager`:

```
GroupsManager {
    groups: Vec<GroupDef>,       // name, order, collapsed state
    assignments: HashMap<String, String>,  // host_name -> group_name
    file_path: PathBuf,
}
```

Introduce a display model concept in `App`:

```
enum DisplayRow {
    GroupHeader { name: String, host_count: usize, collapsed: bool },
    Host(SshHost),
}
```

`App` gains:
- `groups: GroupsManager`
- `display_rows: Vec<DisplayRow>` -- rebuilt on every filter/sort/collapse change
- A method `rebuild_display_rows()` that merges `filtered_hosts` with group info

Navigation (`move_up`, `move_down`, `selected_host`) must be updated to skip `GroupHeader` rows when determining which host is "selected", but allow the cursor to land on headers for collapse/expand.

### 5. Files Impacted

| File | Change | Complexity |
|---|---|---|
| `src/groups.rs` (NEW) | GroupsManager: load/save/CRUD for groups and assignments | Low |
| `src/ui/app.rs` | Add `GroupsManager`, `DisplayRow` enum, `display_rows` vec, `rebuild_display_rows()`, update navigation methods | Medium |
| `src/ui/views/list.rs` | Render `DisplayRow::GroupHeader` as styled non-data rows; adjust row building loop | Medium |
| `src/ui/event.rs` | Add key handlers for group operations (G, g, collapse/expand); update Enter behavior on header rows | Medium |
| `src/main.rs` | Load `GroupsManager` at startup | Low |
| `src/config/mod.rs` | Re-export if needed | Low |

No changes needed to `src/config/parser.rs` -- the SSH config format is untouched.

Estimated total: ~400-600 lines of new/modified code across 5-6 files.

## Alternatives Considered

### Alternative A: Store groups in SSH config via `# Group: name` comments

**Pros:**
- Single source of truth -- group info travels with the config file
- Consistent with how tags are already stored (`# Tags:`)
- No extra config file to manage

**Cons:**
- Pollutes `~/.ssh/config` with sshm-specific metadata that other tools ignore
- Tags are lightweight (metadata about the host), but groups are UI/organizational state (collapsed, order) that does not belong in SSH config
- Requires modifying the parser and the add/update/delete flows
- Group ordering and collapsed state cannot be stored in a comment naturally
- If user edits SSH config by hand, they must maintain the comment convention
- Included config files complicate group assignment (which file to write to?)

### Alternative B: Tree widget with indentation (like a file explorer)

**Pros:**
- Familiar UX for developers (VS Code, file managers)
- Could support nested groups (groups within groups)

**Cons:**
- ratatui `Table` widget does not natively support tree indentation with column alignment
- Would require replacing the Table with a custom widget or `tui-tree-widget` crate (new dependency)
- Nested groups add complexity without clear user need at this stage
- Column alignment breaks when rows have different indent levels

### Alternative C: Accordion tabs above the table (one group visible at a time)

**Pros:**
- Clean separation between groups
- Simple mental model

**Cons:**
- Cannot see hosts from multiple groups simultaneously
- Requires a completely different layout approach
- Poor UX when user wants to multi-select across groups
- Wastes vertical space on tab headers

## Consequences

**Positive:**
- Clean separation of concerns: SSH config stays pure, sshm-rs metadata stays in its own directory
- Follows established pattern (favorites.json, snippets.json)
- No new crate dependencies needed
- Groups are optional -- zero groups means the app works exactly as before
- Flat fallback during search/filter keeps the fast fuzzy search UX intact

**Trade-offs:**
- Group assignments are by host name, so renaming a host in SSH config breaks the assignment (same limitation as favorites -- acceptable, can be documented)
- A host can belong to only one group (not a tree). This is intentional simplicity -- tags already serve the "multiple categories" use case
- The `display_rows` intermediary adds a rebuild step on every filter change, but with typical host counts (<1000) this is negligible

**Risks:**
- Navigation logic becomes more complex with mixed row types (header vs host). Thorough testing of edge cases (empty groups, all collapsed, single host) is essential
- Mouse click handling in the table must account for header rows shifting indices

## Date
2026-03-05
