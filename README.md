# spores

A command-line tool for managing Spotify playlists. Search the Spotify catalog and perform CRUD operations on your playlists, all from the terminal. Every command outputs structured JSON, making it easy to pipe into tools like `jq`.

## Installation

```bash
cargo install --git https://github.com/hedonhermdev/spores.git
```

Or build from source:

```bash
git clone https://github.com/hedonhermdev/spores.git
cd spores
cargo build --release
```

## Setup

1. Register a Spotify application at the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Set the redirect URI to `http://127.0.0.1:8888/callback` in your app settings
3. Run any `spores` command -- it will create a config file and exit with instructions on first run
4. Fill in your `client_id` and `client_secret` in the config file:
   - **macOS**: `~/Library/Application Support/spores/config.toml`
   - **Linux**: `~/.config/spores/config.toml`
5. Run again -- your browser will open for Spotify OAuth authorization. The token is cached automatically for subsequent runs.

### Config format

```toml
client_id = "your_spotify_client_id"
client_secret = "your_spotify_client_secret"
# redirect_uri = "http://127.0.0.1:8888/callback"  # optional, this is the default
```

## Usage

### Search

Search the Spotify catalog for tracks, albums, artists, or playlists.

```bash
# Search for tracks (default)
spores search "Bohemian Rhapsody"

# Search for artists
spores search "Queen" -t artist

# Search for albums with a result limit
spores search "A Night at the Opera" -t album -l 5

# Search for playlists
spores search "chill vibes" -t playlist
```

**Options:**

| Flag | Description |
|---|---|
| `-t, --type <TYPE>` | Type of item: `track` (default), `album`, `artist`, `playlist` |
| `-l, --limit <LIMIT>` | Maximum number of results (default: 20) |

### Playlist management

```bash
# List all your playlists
spores playlist list

# Create a new private playlist
spores playlist create "My Playlist"

# Create a public playlist with a description
spores playlist create "Road Trip" --public -d "Songs for the open road"

# View playlist details and tracks
spores playlist info <playlist_id>

# Add tracks to a playlist
spores playlist add <playlist_id> <track_id1> <track_id2>
```

All playlist and track arguments accept either raw Spotify IDs or full URIs (e.g. `spotify:track:6rqhFgbbKwnb9MLmUQDhG6`).

## License

MIT
