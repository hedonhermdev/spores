---
name: spores
description: Manage Spotify playlists and search the Spotify catalog using the spores Rust CLI. Use this skill when the user asks to create, list, inspect, or modify Spotify playlists, add tracks to playlists, or search for tracks, albums, artists, or playlists on Spotify. Triggers on requests like "create a Spotify playlist", "add songs to my playlist", "search Spotify for...", "list my playlists", "what tracks are in this playlist", or any Spotify playlist management task.
---

# Spores

Spores is a Rust CLI tool (`spores`) for Spotify playlist management. All output is structured JSON. The entire codebase lives in `src/main.rs` (~480 lines).

## Prerequisites

Before running any command, the user must have:

1. A Spotify developer app registered at https://developer.spotify.com/dashboard with a redirect URI of `http://127.0.0.1:8888/callback`
2. A config file at the platform config directory (`~/Library/Application Support/spores/config.toml` on macOS, `~/.config/spores/config.toml` on Linux) containing:

```toml
client_id = "<SPOTIFY_CLIENT_ID>"
client_secret = "<SPOTIFY_CLIENT_SECRET>"
# redirect_uri = "http://127.0.0.1:8888/callback"  # optional, this is the default
```

On first run, spores opens a browser for OAuth authorization and caches the token at `config_dir/token_cache.json`. Subsequent runs reuse the cached token.

If the config file does not exist, `spores` creates a template and exits with instructions. If credentials are empty, it exits with an error.

## CLI Command Reference

All commands are run via `cargo run -- <subcommand>` during development or `spores <subcommand>` if installed.

### Search

```
spores search <QUERY> [-t track|album|artist|playlist] [-l LIMIT]
```

- `QUERY` (required): search string
- `-t, --type`: item type to search for (default: `track`)
- `-l, --limit`: max results (default: `20`)

Output shape varies by type. Track example:

```json
{
  "query": "bohemian rhapsody",
  "type": "track",
  "total": 857,
  "items": [
    {
      "id": "7tFiyTwD0nx5a1eklYtX2J",
      "name": "Bohemian Rhapsody",
      "artists": ["Queen"],
      "album": "A Night At The Opera",
      "duration_ms": 354947
    }
  ]
}
```

Album items have: `id`, `name`, `artists`, `release_date`.
Artist items have: `id`, `name`, `genres`, `followers`, `popularity`.
Playlist items have: `id`, `name`, `tracks`, `owner`, `url`.

### Playlist List

```
spores playlist list
```

Lists all playlists for the authenticated user (paginates automatically).

```json
{
  "total": 12,
  "playlists": [
    {
      "id": "37i9dQZF1DXcBWIGoYBM5M",
      "name": "Today's Top Hits",
      "tracks": 50,
      "public": true,
      "owner": "spotify",
      "url": "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M"
    }
  ]
}
```

### Playlist Create

```
spores playlist create <NAME> [--public] [-d DESCRIPTION]
```

- `NAME` (required): playlist name
- `--public`: make playlist public (default: private)
- `-d, --description`: optional description

```json
{
  "id": "3cEYpjA9oz9GiPac4AsH4n",
  "name": "Road Trip",
  "public": false,
  "description": "Songs for the drive",
  "url": "https://open.spotify.com/playlist/3cEYpjA9oz9GiPac4AsH4n"
}
```

### Playlist Info

```
spores playlist info <PLAYLIST>
```

- `PLAYLIST` (required): playlist ID or Spotify URI

Returns full playlist details including all tracks:

```json
{
  "id": "3cEYpjA9oz9GiPac4AsH4n",
  "name": "Road Trip",
  "owner": "tirth",
  "public": false,
  "collaborative": false,
  "followers": 0,
  "description": "Songs for the drive",
  "url": "https://open.spotify.com/playlist/3cEYpjA9oz9GiPac4AsH4n",
  "total_tracks": 2,
  "tracks": [
    {
      "type": "track",
      "id": "7tFiyTwD0nx5a1eklYtX2J",
      "name": "Bohemian Rhapsody",
      "artists": ["Queen"],
      "album": "A Night At The Opera",
      "duration_ms": 354947
    }
  ]
}
```

Tracks may also have `"type": "episode"` (with `show` and `duration_ms`) or `"type": "unknown"`.

### Playlist Add

```
spores playlist add <PLAYLIST> <TRACKS>...
```

- `PLAYLIST` (required): playlist ID or Spotify URI
- `TRACKS` (required): one or more track IDs or Spotify URIs

```json
{
  "playlist": "3cEYpjA9oz9GiPac4AsH4n",
  "added": 2,
  "snapshot_id": "MTYsNjM..."
}
```

## Workflow: Creating a Playlist and Adding Tracks

1. Search for tracks to find their IDs:
   ```
   cargo run -- search "bohemian rhapsody" -t track -l 5
   ```
2. Extract the `id` values from the JSON output.
3. Create a new playlist:
   ```
   cargo run -- playlist create "My Playlist" -d "A great playlist"
   ```
4. Extract the playlist `id` from the output.
5. Add tracks to the playlist:
   ```
   cargo run -- playlist add <PLAYLIST_ID> <TRACK_ID_1> <TRACK_ID_2>
   ```
6. Verify with:
   ```
   cargo run -- playlist info <PLAYLIST_ID>
   ```

## Workflow: Inspecting Existing Playlists

1. List all playlists: `cargo run -- playlist list`
2. Pick a playlist ID from the output.
3. Get full details: `cargo run -- playlist info <PLAYLIST_ID>`

## Key Implementation Details

- **IDs and URIs**: All commands accepting playlist or track identifiers accept both raw IDs (e.g., `7tFiyTwD0nx5a1eklYtX2J`) and full Spotify URIs (e.g., `spotify:track:7tFiyTwD0nx5a1eklYtX2J`).
- **JSON output**: Every command outputs pretty-printed JSON via `print_json()`. Errors are also JSON: `{"error": "message"}`.
- **Auth scopes**: `playlist-read-private`, `playlist-read-collaborative`, `playlist-modify-public`, `playlist-modify-private`.
- **Pagination**: `playlist list` automatically pages through all results (offset/limit=50).
- **Error handling**: The codebase uses `.unwrap()` throughout. Config/startup errors go to stderr via `eprintln!()` + `process::exit(1)`.

## Adding New Commands

To extend spores with a new command:

1. Add a variant to the `Command` enum (for top-level) or `PlaylistCommand` enum (for playlist subcommands) in the CLI section of `src/main.rs`.
2. Write an `async fn cmd_<name>(spotify: &AuthCodeSpotify, ...)` handler following the existing pattern.
3. Build output with `serde_json::json!({...})` and call `print_json(&value)`.
4. Add a match arm in `main()` to dispatch to the handler.
5. Run `cargo build` to verify, `cargo fmt` to format, `cargo clippy -- -D warnings` to lint.

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` (4, derive) | CLI argument parsing |
| `rspotify` (0.15.3, cli) | Spotify Web API client with OAuth |
| `serde` / `serde_json` | JSON serialization |
| `tokio` (full) | Async runtime |
| `toml` (0.8) | Config file parsing |
| `dirs` (6) | Platform config directory resolution |

Do not add new dependencies without clear justification.
