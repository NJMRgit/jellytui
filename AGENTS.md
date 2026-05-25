# AGENTS.md — jellytui

## Build & run

```bash
cargo build --release
./target/release/jellytui
```

**Minimum Rust**: 1.85 (edition 2024 in `Cargo.toml`).

## Architecture

Single binary crate. Entry: `src/main.rs`. Internal modules:
- `app` — state machine, navigation, login flow, playback/dnload tracking
- `client` — Jellyfin REST client (auth, items, search, playback reporting)
- `config` — TOML config read/write (`~/.config/jellytui/config.toml`)
- `player` — spawns `mpv` process, IPC over Unix socket, monitors playback
- `download` — HTTP streaming download with progress events
- `events` — crossterm key/tick event loop on tokio
- `ui` — ratatui rendering (login, browser, search, now-playing, downloads)

Screens flow: Login → Home → Library (nav stack) ↔ Search (overlay).

## System dependencies

**mpv** must be installed and on `$PATH`. No startup check — if missing, playback fails at runtime.

MPV is spawned per playback session with a temp Unix socket (`/tmp/jellytui-mpv-{pid}-{uuid}.sock`). MPV logs go to `~/.config/jellytui/logs/mpv.log`.

On Wayland, mpv gets `--gpu-context=waylandvk` automatically (detected via `WAYLAND_DISPLAY` env).

## Conventions

- **Auth header**: Uses `X-Emby-Authorization` (Jellyfin's Emby-compat header), NOT `Authorization: Bearer`.
- **Config**: `server_url`, `access_token`, `user_id` in TOML. Saved on successful login; cleared on 401.
- **Playback session IDs**: `playback_session_id` (u64, wrapping_add) is used as a generation counter — stale PlayerEvents are silently dropped if their session_id doesn't match.
- **Tick-based**: Tick event every 250ms via `EventHandler`. Position reporting every 5s.
- **Search**: Typing triggers `perform_search()` on each keystroke (no debounce). Enter on a result or press `Esc` to close.
- **Downloads**: `d` toggles popup; `D` queues download. Files go to `~/Downloads/jellytui/`.
- **Key handling**: `Ctrl+C` quits same as `q`. `BackTab` + `Shift+Tab` both cycle login fields backward.

## Error handling quirk

Every API call in `load_home_content()` independently catches errors and calls `handle_unauthorized()`. A 401 resets state to the login screen, clears the token from config, and saves. Other errors accumulate but only the first sets `error_message`.

## No CI / tests

The repo has no CI workflows, no unit tests, no integration tests, and no lint/format config (no `rustfmt.toml`, no `clippy.toml`). There's nothing to configure before submitting changes.
