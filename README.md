# Jellytui

A terminal UI client for [Jellyfin](https://jellyfin.org/) written in Rust.

<img width="1366" height="768" alt="image" src="https://github.com/user-attachments/assets/13542204-4071-4562-a905-8fa6c73f0f19" />


## Features

- Browse libraries and navigate folders
- Search across all media
- Play with MPV 
- Resume from last position
- Download media files
- Sidebar with selection information and poster/preview
- SVP (smooth Video Project) Support
- Playback Sync:
  - Resume from where you left off
  - Progress reported every 5 seconds
  - Auto-marks as played at 90%
- Fallback Image Rendering

<img width="643" height="496" alt="image" src="https://github.com/user-attachments/assets/21e4529e-3148-408b-aca6-b4e92cf941b1" />

- compact mode

<img width="248" height="366" alt="image" src="https://github.com/user-attachments/assets/90d0d74d-df14-49f6-9603-481b18d5b68b" />




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
### Downloads

Files are saved to `~/Downloads/jellytui/`.

### Keybindings

| Key | Action |
|-----|--------|
| `↓` / `j` | Move down |
| `↑` / `k` | Move up |
| `→` / `l` / `ENTER` | Open / Play |
| `←` /`Esc` / `h` | Go back |
| `/` or `s` | Search |
| `d` | Toggle downloads panel |
| `D` | Download selected item |
| `r` | Refresh |
| `q` | Quit |

## Disclaimer

I used opencode/pi for this

## License

MIT
