# godot-updater

A terminal-based Godot version manager. Browse, download, install, and launch Godot Engine releases without leaving your terminal.

## What it does

- Fetches available Godot releases directly from GitHub (stable, preview/dev, and LTS channels)
- Shows which versions you already have installed by scanning your install directory
- Downloads and extracts new versions with a progress bar
- Launches any installed version from the TUI
- Tracks installations with a local manifest (`installations.json`)
- Supports Standard and Mono (C#) editions
- Works on Windows, Linux, and macOS

## Building

Requires Rust 1.70+ and Cargo.

```
git clone <repo-url>
cd godot-updater
cargo build --release
```

The binary will be at `target/release/godot-updater` (or `godot-updater.exe` on Windows).

## Usage

```
cargo run --release
```

Or run the binary directly after building.

### Keybindings

| Key | Action |
|-----|--------|
| Up/Down, j/k | Navigate version list |
| Enter | Download and install the selected version |
| L | Launch the selected installed version |
| D | Delete/uninstall the selected version |
| R | Refresh the version list from GitHub |
| Tab | Cycle channel filter (All, Stable, Preview, LTS) |
| E | Toggle edition (Standard / Mono) |
| F2 | Open settings |
| Home/End | Jump to first/last version |
| Q | Quit |

### Settings (F2)

- **Install Directory** - where Godot versions are stored (free text, press Enter to edit)
- **Edition** - toggle between `standard`, `mono`, or `both`
- **Stable/Preview/LTS Channels** - toggle which release channels to fetch
- **Theme** - switch between `Default` and `MagicWB`

All settings are saved to a TOML config file at:
- Windows: `%APPDATA%/godot-updater/config.toml`
- Linux: `~/.config/godot-updater/config.toml`
- macOS: `~/Library/Application Support/godot-updater/config.toml`

### Existing installs

When you set your install directory (or change it in settings), the app scans for existing Godot binaries matching the standard naming pattern (e.g. `Godot_v4.6.1-stable_win64.exe`) and registers them automatically. Versions that are deleted outside the app are cleaned up from the manifest on the next scan.

## Project structure

The project is a Cargo workspace with two crates:

- **godot-updater-core** - library with config management, GitHub API client, version parsing, download/install logic, and platform detection. Designed so a GUI frontend could reuse it.
- **godot-updater** (TUI) - terminal interface built with [Ratatui](https://ratatui.rs) and [crossterm](https://github.com/crossterm-rs/crossterm).

```
godot-updater/
  Cargo.toml              # workspace root
  crates/
    core/src/
      config.rs            # TOML config load/save
      github.rs            # GitHub releases API client
      versions.rs          # Version parsing and ordering
      download.rs          # Async file download with progress
      install.rs           # ZIP extraction, manifest, launch
      platform.rs          # Platform-specific paths and asset names
    tui/src/
      main.rs              # Entry point, terminal setup, async event loop
      app.rs               # App state, key handling, settings logic
      theme.rs             # Color themes (Default, MagicWB)
      events.rs            # Input event polling
      ui/
        versions.rs        # Main version list view
        download.rs        # Download progress view
        settings.rs        # Settings editor view
```

## License

MIT
