# Jellytui Plan

A TUI client for Jellyfin written in Rust.

## Architecture
- **Language:** Rust (2024 edition)
- **UI Framework:** [ratatui](https://crates.io/crates/ratatui) with [crossterm](https://crates.io/crates/crossterm) backend.
- **Async Runtime:** [tokio](https://crates.io/crates/tokio) (for API calls and event handling).
- **API Client:** Direct `reqwest` implementation.
- **Playback:** External player integration (`mpv` primary) via IPC.
- **Config:** `toml` for saving server URL and auth token.

## Phases

### Phase 1: Foundation & Setup [DONE]
- [x] Initialize Rust project structure.
- [x] Add dependencies: `ratatui`, `crossterm`, `tokio`, `anyhow`, `serde`, `serde_json`, `reqwest`.
- [x] Create basic TUI scaffolding (main.rs, app.rs, ui.rs, events.rs).

### Phase 2: Authentication & Configuration [DONE]
- [x] Create `Config` struct to load/save `~/.config/jellytui/config.toml`.
- [x] Implement Login Screen UI (Server URL, Username, Password).
- [x] Implement Auth Logic (authenticate, store token).

### Phase 3: Browsing Core [DONE]
- [x] Implement "Home" view (fetch User Views/Libraries).
- [x] Implement "Library" view (list items, navigate folders).
- [x] Navigation logic (arrows/jk, Enter, Escape).
- [x] Basic MPV playback (spawn with stream URL).

### Phase 4: Playback Sync & MPV IPC [DONE]
- [x] Add `mpv-ipc` crate for IPC communication.
- [x] Report playback start to Jellyfin (`/Sessions/Playing`).
- [x] Report progress periodically (`/Sessions/Playing/Progress`).
- [x] Report playback stop (`/Sessions/Playing/Stopped`).
- [x] Resume from last position (fetch PlaybackPositionTicks).
- [x] Mark as played when >90% watched.

### Phase 5: Search [DONE]
- [x] Global Search Popup (triggered by `/` or `s`).
- [x] Query Jellyfin Search API.
- [x] Display results in a list.
- [x] Navigate to item from results.

### Phase 6: Download & Extras [DONE]
- [x] Download manager task (background tokio task).
- [x] UI for download progress.

## Key Libraries
- `ratatui`
- `tokio`
- `reqwest`
- `crossterm`
- `serde`
- `mpv-ipc` (for playback sync)
