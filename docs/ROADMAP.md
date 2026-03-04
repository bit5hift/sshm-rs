# sshm-rs Roadmap

## Tier 1 — Quick Wins (1-3h each)

| # | Feature | Status |
|---|---------|--------|
| 1 | Wire PingManager (status indicators actually work) | TODO |
| 2 | Edit Host form (`e` key) | TODO |
| 3 | Fix Esc: don't quit app, only `q` quits | TODO |
| 4 | Unicode status indicators (green/red circles) | TODO |
| 5 | Rounded borders (BorderType::Rounded) | TODO |
| 6 | PageUp/PageDown/gg/G navigation | TODO |
| 7 | Toast messages after operations | TODO |
| 8 | Shell completions (clap_complete) | TODO |

## Tier 2 — Differentiators (3-8h each)

| # | Feature | Status |
|---|---------|--------|
| 9 | Fuzzy search (nucleo crate) | TODO |
| 10 | Port Forwarding TUI | TODO |
| 11 | Host grouping / tag filtering (`tag:production`) | TODO |
| 12 | Mouse support (click, scroll wheel) | TODO |
| 13 | Scrollbar widget | TODO |
| 14 | Responsive title (hide ASCII art on small terminals) | TODO |
| 15 | Favorites / pinned hosts | TODO |
| 16 | Extended Tokyo Night palette (cyan, purple, orange) | TODO |

## Tier 3 — Advanced Features (8h+ each)

| # | Feature | Status |
|---|---------|--------|
| 17 | Command broadcast (multi-server exec) | TODO |
| 18 | Snippet / command templates | TODO |
| 19 | SFTP/SCP integration | TODO |
| 20 | Multi-select + batch operations | TODO |
| 21 | Export/Import configs | TODO |
| 22 | Clipboard support (`y` to copy hostname) | TODO |
| 23 | Configurable themes (high-contrast, light, custom) | TODO |
| 24 | Config validation | TODO |

## Tier 4 — Long-term Vision

- Auto-reconnection / session persistence (mosh-like)
- Cloud sync without subscription (Git-based?)
- AWS/GCP/Azure host discovery
- SSH key management UI (generate, rotate)
- FIDO2/YubiKey support
- Session recording / audit log
- Split-pane multi-session

## Research Sources

- Competitive analysis: Termius, MobaXterm, Tabby, Warp, sshs, lazyssh, storm, XPipe
- Community: Reddit (r/sysadmin, r/homelab, r/devops), Hacker News, GitHub Issues
- UX audit: comparison with lazygit, k9s, bottom TUI patterns
