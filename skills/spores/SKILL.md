---
name: spores
description: Manage and develop the spores Rust CLI tool for Spotify playlist management. Use this skill when the user asks to build features, fix bugs, or extend the spores CLI, or when working with Spotify Web API integration via rspotify in this project.
---

# Spores

## Overview

Spores is a Rust CLI tool that manages Spotify playlists via the Spotify Web API. It uses `rspotify` for API interaction with OAuth Authorization Code flow, `clap` (derive API) for subcommands, and outputs all results as pretty-printed JSON.

## Installation

```bash
cargo install --git https://github.com/hedonhermdev/spores.git
```

This places the `spores` binary on your `$PATH` (typically `~/.cargo/bin/spores`).

### First-Run Setup

On first run, spores auto-creates a config file at `$XDG_CONFIG_HOME/spores/config.toml` (macOS: `~/Library/Application Support/spores/config.toml`) and exits with instructions.

1. Create a Spotify app at https://developer.spotify.com/dashboard
2. Set the redirect URI in the Spotify dashboard to `http://127.0.0.1:8888/callback`
3. Fill in the generated config file:

```toml
client_id = "your_client_id"
client_secret = "your_client_secret"
# redirect_uri = "http://127.0.0.1:8888/callback"  # optional, this is the default
```

4. Run any command (e.g. `spores playlist list`) — the browser opens for OAuth authorization. The token is cached at `$XDG_CONFIG_HOME/spores/token_cache.json` and reused/refreshed automatically on subsequent runs.

## Architecture

### Configuration

Credentials and settings are loaded from `$XDG_CONFIG_HOME/spores/config.toml` using the `dirs`, `toml`, and `serde` crates. The `AppConfig` struct is deserialized from TOML with fields: `client_id`, `client_secret`, and optional `redirect_uri` (defaults to `http://127.0.0.1:8888/callback`).

The `config_dir()` helper returns the path via `dirs::config_dir().join("spores")`.

### Authentication

OAuth Authorization Code flow via `rspotify::AuthCodeSpotify`. Credentials are constructed manually with `Credentials::new()`. The `cli` feature provides `prompt_for_token()` which opens the browser for user authorization.

Token caching is enabled via `rspotify::Config { token_cached: true, cache_path }` pointing to `$XDG_CONFIG_HOME/spores/token_cache.json`. On subsequent runs, rspotify loads and refreshes the cached token automatically — no browser interaction needed.

Required scopes: `playlist-read-private`, `playlist-read-collaborative`, `playlist-modify-public`, `playlist-modify-private`.

The redirect URI must use `127.0.0.1` (not `localhost`) because Spotify rejects localhost.

### CLI Structure

The CLI uses nested subcommands. Top-level commands are `search` and `playlist`. All playlist operations are subcommands of `playlist`.

#### `search <query> [-t type] [-l limit]`

Searches Spotify. The `-t`/`--type` flag accepts `track` (default), `album`, `artist`, or `playlist`. The `-l`/`--limit` flag controls max results (default 20). Uses `rspotify::model::SearchType` and returns `SearchResult` — match on the variant corresponding to the search type.

#### `playlist list`

Lists all user playlists with pagination (50 per page via `current_user_playlists_manual`).

#### `playlist create <name> [--public] [-d description]`

Creates a new playlist for the current user.

#### `playlist info <playlist>`

Shows full playlist details including tracks (accepts playlist ID or Spotify URI).

#### `playlist add <playlist> <tracks...>`

Adds one or more tracks to a playlist (accepts track IDs or URIs).

### Output Convention

All output is JSON via `serde_json::to_string_pretty`. A shared `print_json(&Value)` helper is used throughout.

## Development Guide

### Key rspotify Patterns

- Use `rspotify::model` for types: `PlaylistId`, `TrackId`, `UserId`, `PlayableId`, `PlayableItem`, `SearchType`, `SearchResult`, `FullPlaylist`, `SimplifiedPlaylist`
- `PlaylistId::from_id_or_uri()` and `TrackId::from_id_or_uri()` accept both raw IDs and full Spotify URIs
- `PlayableItem` is an enum with `Track`, `Episode`, and `Unknown(_)` variants — always include a wildcard arm when matching
- `current_user_playlists_manual` takes `Option<u32>` for limit/offset, returns `Page<SimplifiedPlaylist>` — check `.next.is_none()` for pagination end
- `playlist_add_items` returns `PlaylistResult` with a `snapshot_id` field

### Building and Running

```bash
# Development
cargo build
cargo run -- search "Bohemian Rhapsody"
cargo run -- search "Queen" -t artist -l 5
cargo run -- playlist list
cargo run -- playlist create "My Playlist" --public -d "A great playlist"
cargo run -- playlist info <playlist_id_or_uri>
cargo run -- playlist add <playlist_id_or_uri> <track_id_or_uri> [more_tracks...]

# Install globally
cargo install --git https://github.com/hedonhermdev/spores.git
spores search "Bohemian Rhapsody"
spores playlist list
```

### Adding New Commands

**Top-level command:** Add a variant to the `Command` enum, write the handler, add a match arm in `main()`.

**Playlist subcommand:** Add a variant to the `PlaylistCommand` enum, write the handler, add a match arm in the `Command::Playlist` branch in `main()`.

### Common API Methods (rspotify)

- `spotify.search(query, search_type, market, include_external, limit, offset)` — Search Spotify catalog
- `spotify.current_user()` — Get current user profile
- `spotify.current_user_playlists_manual(limit, offset)` — Paginated playlist listing
- `spotify.user_playlist_create(user_id, name, public, collaborative, description)` — Create playlist
- `spotify.playlist(playlist_id, fields, market)` — Get full playlist details
- `spotify.playlist_add_items(playlist_id, items, position)` — Add tracks to playlist
- `spotify.playlist_remove_all_occurrences_of_items(playlist_id, items, snapshot_id)` — Remove tracks
- `spotify.playlist_reorder_items(playlist_id, range_start, range_length, insert_before, snapshot_id)` — Reorder tracks

### Rust Edition

This project uses Rust edition **2024**.
