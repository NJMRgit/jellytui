# Jellytui

A terminal UI client for [Jellyfin](https://jellyfin.org/) written in Rust.

## Features

- Browse libraries and navigate folders
- Search across all media
- Play with MPV (with playback sync)
- Resume from last position
- Download media files

## Requirements

- Rust 1.85+ (2024 edition)
- MPV (for playback)
- A Jellyfin server

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/jellytui
```

## Usage

```bash
jellytui
```

On first run, enter your Jellyfin server URL and credentials. Config is saved to `~/.config/jellytui/config.toml`.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` / `l` | Open / Play |
| `Esc` / `h` | Go back |
| `/` or `s` | Search |
| `d` | Toggle downloads panel |
| `D` | Download selected item |
| `r` | Refresh |
| `q` | Quit |

### Downloads

Files are saved to `~/Downloads/jellytui/`.

## Playback Sync

Playback position is synced to Jellyfin:
- Resume from where you left off
- Progress reported every 5 seconds
- Auto-marks as played at 90%

## License

MIT
