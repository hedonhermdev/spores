use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};
use rspotify::{
    AuthCodeSpotify, Config as SpotifyConfig, Credentials, OAuth,
    model::{AlbumId, PlaylistId, SearchResult, SearchType, TrackId, UserId},
    prelude::*,
    scopes,
};
use serde::Deserialize;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct AppConfig {
    client_id: String,
    client_secret: String,
    redirect_uri: Option<String>,
}

fn config_dir() -> PathBuf {
    let base = dirs::config_dir().expect("could not determine XDG_CONFIG_HOME");
    base.join("spores")
}

fn load_config() -> AppConfig {
    let dir = config_dir();
    let path = dir.join("config.toml");

    if !path.exists() {
        fs::create_dir_all(&dir).expect("failed to create config directory");
        let template = r#"# Spotify application credentials
# Create an app at https://developer.spotify.com/dashboard
client_id = ""
client_secret = ""

# Must match the redirect URI registered in your Spotify app.
# Use 127.0.0.1 — Spotify rejects "localhost".
# redirect_uri = "http://127.0.0.1:8888/callback"
"#;
        fs::write(&path, template).expect("failed to write default config");
        eprintln!("Created config file at {}", path.display());
        eprintln!("Please fill in your Spotify credentials and run again.");
        process::exit(1);
    }

    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    let config: AppConfig = toml::from_str(&contents)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));

    if config.client_id.is_empty() || config.client_secret.is_empty() {
        eprintln!(
            "client_id and client_secret must be set in {}",
            path.display()
        );
        process::exit(1);
    }

    config
}

fn prompt(label: &str, default: Option<&str>) -> String {
    let mut stdout = io::stdout().lock();
    if let Some(d) = default {
        write!(stdout, "{label} [{d}]: ").unwrap();
    } else {
        write!(stdout, "{label}: ").unwrap();
    }
    stdout.flush().unwrap();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    let trimmed = buf.trim().to_string();

    if trimmed.is_empty() {
        default.unwrap_or("").to_string()
    } else {
        trimmed
    }
}

fn cmd_configure() {
    let dir = config_dir();
    let path = dir.join("config.toml");

    println!("Spores configuration wizard");
    println!("Create a Spotify app at https://developer.spotify.com/dashboard");
    println!();

    // Load existing config values as defaults if the file already exists.
    let existing: Option<AppConfig> = path
        .exists()
        .then(|| {
            let contents = fs::read_to_string(&path).ok()?;
            toml::from_str(&contents).ok()
        })
        .flatten();

    let default_id = existing.as_ref().and_then(|c| {
        if c.client_id.is_empty() {
            None
        } else {
            Some(c.client_id.as_str())
        }
    });
    let default_secret = existing.as_ref().and_then(|c| {
        if c.client_secret.is_empty() {
            None
        } else {
            Some(c.client_secret.as_str())
        }
    });
    let default_redirect = existing
        .as_ref()
        .and_then(|c| c.redirect_uri.as_deref())
        .or(Some("http://127.0.0.1:8888/callback"));

    let client_id = prompt("Client ID", default_id);
    let client_secret = prompt("Client secret", default_secret);
    let redirect_uri = prompt("Redirect URI", default_redirect);

    if client_id.is_empty() || client_secret.is_empty() {
        eprintln!("client_id and client_secret are required.");
        process::exit(1);
    }

    fs::create_dir_all(&dir).expect("failed to create config directory");

    let config_toml = format!(
        r#"# Spotify application credentials
# Create an app at https://developer.spotify.com/dashboard
client_id = "{client_id}"
client_secret = "{client_secret}"

# Must match the redirect URI registered in your Spotify app.
# Use 127.0.0.1 — Spotify rejects "localhost".
redirect_uri = "{redirect_uri}"
"#
    );

    fs::write(&path, config_toml).expect("failed to write config file");
    println!();
    println!("Configuration saved to {}", path.display());
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "spores", about = "Spotify playlist manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search Spotify
    Search {
        /// Search query
        query: String,

        /// Type of item to search for
        #[arg(short, long, value_enum, default_value_t = ItemType::Track)]
        r#type: ItemType,

        /// Maximum number of results
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },

    /// Manage playlists
    Playlist {
        #[command(subcommand)]
        command: PlaylistCommand,
    },

    /// Configure Spotify credentials interactively
    Configure,

    /// Save a track, album, or playlist to your library
    Save {
        /// Type of item to save
        #[arg(short, long, value_enum, default_value_t = ItemType::Track)]
        r#type: ItemType,

        /// IDs or URIs of items to save
        #[arg(required = true)]
        ids: Vec<String>,
    },
}

#[derive(Subcommand)]
enum PlaylistCommand {
    /// List your playlists
    List,

    /// Create a new playlist
    Create {
        /// Name of the playlist
        name: String,

        /// Make the playlist public
        #[arg(long)]
        public: bool,

        /// Description for the playlist
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Show details of a playlist
    Info {
        /// Playlist ID or URI
        playlist: String,
    },

    /// Add tracks to a playlist
    Add {
        /// Playlist ID or URI
        playlist: String,

        /// Track IDs or URIs to add
        #[arg(required = true)]
        tracks: Vec<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum ItemType {
    Track,
    Album,
    Artist,
    Playlist,
}

impl ItemType {
    fn to_search_type(&self) -> SearchType {
        match self {
            ItemType::Track => SearchType::Track,
            ItemType::Album => SearchType::Album,
            ItemType::Artist => SearchType::Artist,
            ItemType::Playlist => SearchType::Playlist,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

// ---------------------------------------------------------------------------
// Auth
// ---------------------------------------------------------------------------

async fn authenticate() -> AuthCodeSpotify {
    let app = load_config();

    let creds = Credentials::new(&app.client_id, &app.client_secret);

    let redirect_uri = app
        .redirect_uri
        .unwrap_or_else(|| "http://127.0.0.1:8888/callback".to_string());

    let oauth = OAuth {
        redirect_uri,
        scopes: scopes!(
            "playlist-read-private",
            "playlist-read-collaborative",
            "playlist-modify-public",
            "playlist-modify-private",
            "user-library-modify"
        ),
        ..Default::default()
    };

    let cache_path = config_dir().join("token_cache.json");

    let config = SpotifyConfig {
        token_cached: true,
        cache_path,
        ..Default::default()
    };

    let spotify = AuthCodeSpotify::with_config(creds, oauth, config);
    let url = spotify.get_authorize_url(false).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();
    spotify
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

async fn cmd_search(spotify: &AuthCodeSpotify, query: &str, item_type: &ItemType, limit: u32) {
    let search_type = item_type.to_search_type();
    let result = spotify
        .search(query, search_type, None, None, Some(limit), None)
        .await
        .unwrap();

    match result {
        SearchResult::Tracks(page) => {
            let tracks: Vec<Value> = page
                .items
                .iter()
                .map(|track| {
                    let artists: Vec<&str> =
                        track.artists.iter().map(|a| a.name.as_str()).collect();
                    json!({
                        "id": track.id.as_ref().map(|id| id.to_string()),
                        "name": track.name,
                        "artists": artists,
                        "album": track.album.name,
                        "duration_ms": track.duration.num_milliseconds(),
                    })
                })
                .collect();

            print_json(&json!({
                "query": query,
                "type": "track",
                "total": page.total,
                "items": tracks,
            }));
        }
        SearchResult::Albums(page) => {
            let albums: Vec<Value> = page
                .items
                .iter()
                .map(|album| {
                    let artists: Vec<&str> =
                        album.artists.iter().map(|a| a.name.as_str()).collect();
                    json!({
                        "id": album.id.as_ref().map(|id| id.to_string()),
                        "name": album.name,
                        "artists": artists,
                        "release_date": album.release_date,
                    })
                })
                .collect();

            print_json(&json!({
                "query": query,
                "type": "album",
                "total": page.total,
                "items": albums,
            }));
        }
        SearchResult::Artists(page) => {
            let artists: Vec<Value> = page
                .items
                .iter()
                .map(|artist| {
                    json!({
                        "id": artist.id.to_string(),
                        "name": artist.name,
                        "genres": artist.genres,
                        "followers": artist.followers.total,
                        "popularity": artist.popularity,
                    })
                })
                .collect();

            print_json(&json!({
                "query": query,
                "type": "artist",
                "total": page.total,
                "items": artists,
            }));
        }
        SearchResult::Playlists(page) => {
            let playlists: Vec<Value> = page
                .items
                .iter()
                .map(|playlist| {
                    json!({
                        "id": playlist.id.to_string(),
                        "name": playlist.name,
                        "tracks": playlist.tracks.total,
                        "owner": playlist.owner.display_name.as_deref().unwrap_or("unknown"),
                        "url": playlist.external_urls.get("spotify"),
                    })
                })
                .collect();

            print_json(&json!({
                "query": query,
                "type": "playlist",
                "total": page.total,
                "items": playlists,
            }));
        }
        _ => {
            print_json(&json!({ "error": "unsupported search type" }));
        }
    }
}

// ---------------------------------------------------------------------------
// Playlist commands
// ---------------------------------------------------------------------------

async fn cmd_playlist_list(spotify: &AuthCodeSpotify) {
    let mut playlists: Vec<Value> = Vec::new();
    let mut offset = 0u32;
    let limit = 50u32;

    loop {
        let page = spotify
            .current_user_playlists_manual(Some(limit), Some(offset))
            .await
            .unwrap();

        for playlist in &page.items {
            playlists.push(json!({
                "id": playlist.id.to_string(),
                "name": playlist.name,
                "tracks": playlist.tracks.total,
                "public": playlist.public.unwrap_or(false),
                "owner": playlist.owner.display_name.as_deref().unwrap_or("unknown"),
                "url": playlist.external_urls.get("spotify"),
            }));
        }

        if page.next.is_none() {
            break;
        }

        offset += limit;
    }

    print_json(&json!({
        "total": playlists.len(),
        "playlists": playlists,
    }));
}

async fn cmd_playlist_create(
    spotify: &AuthCodeSpotify,
    name: &str,
    public: bool,
    description: Option<&str>,
) {
    let user = spotify.current_user().await.unwrap();
    let user_id = UserId::from_id(user.id.id()).unwrap();

    let playlist = spotify
        .user_playlist_create(user_id, name, Some(public), None, description)
        .await
        .unwrap();

    print_json(&json!({
        "id": playlist.id.to_string(),
        "name": playlist.name,
        "public": playlist.public.unwrap_or(false),
        "description": playlist.description,
        "url": playlist.external_urls.get("spotify"),
    }));
}

async fn cmd_playlist_info(spotify: &AuthCodeSpotify, playlist_id_str: &str) {
    let playlist_id = PlaylistId::from_id_or_uri(playlist_id_str).unwrap();
    let playlist = spotify.playlist(playlist_id, None, None).await.unwrap();

    let tracks: Vec<Value> = playlist
        .tracks
        .items
        .iter()
        .filter_map(|item| {
            item.track.as_ref().map(|playable| match playable {
                rspotify::model::PlayableItem::Track(track) => {
                    let artists: Vec<&str> =
                        track.artists.iter().map(|a| a.name.as_str()).collect();
                    json!({
                        "type": "track",
                        "id": track.id.as_ref().map(|id| id.to_string()),
                        "name": track.name,
                        "artists": artists,
                        "album": track.album.name,
                        "duration_ms": track.duration.num_milliseconds(),
                    })
                }
                rspotify::model::PlayableItem::Episode(episode) => {
                    json!({
                        "type": "episode",
                        "id": episode.id.to_string(),
                        "name": episode.name,
                        "show": episode.show.name,
                        "duration_ms": episode.duration.num_milliseconds(),
                    })
                }
                _ => json!({ "type": "unknown" }),
            })
        })
        .collect();

    print_json(&json!({
        "id": playlist.id.to_string(),
        "name": playlist.name,
        "owner": playlist.owner.display_name.as_deref().unwrap_or("unknown"),
        "public": playlist.public.unwrap_or(false),
        "collaborative": playlist.collaborative,
        "followers": playlist.followers.total,
        "description": playlist.description,
        "url": playlist.external_urls.get("spotify"),
        "total_tracks": playlist.tracks.total,
        "tracks": tracks,
    }));
}

async fn cmd_playlist_add(spotify: &AuthCodeSpotify, playlist_id_str: &str, track_strs: &[String]) {
    let playlist_id = PlaylistId::from_id_or_uri(playlist_id_str).unwrap();

    let track_ids: Vec<TrackId<'_>> = track_strs
        .iter()
        .map(|t| TrackId::from_id_or_uri(t).unwrap())
        .collect();

    let playable_ids: Vec<rspotify::model::PlayableId<'_>> = track_ids
        .iter()
        .map(|id| rspotify::model::PlayableId::Track(id.clone()))
        .collect();

    let result = spotify
        .playlist_add_items(playlist_id, playable_ids, None)
        .await
        .unwrap();

    print_json(&json!({
        "playlist": playlist_id_str,
        "added": track_strs.len(),
        "snapshot_id": result.snapshot_id,
    }));
}

// ---------------------------------------------------------------------------
// Save to library
// ---------------------------------------------------------------------------

async fn cmd_save(spotify: &AuthCodeSpotify, item_type: &ItemType, ids: &[String]) {
    match item_type {
        ItemType::Track => {
            let track_ids: Vec<TrackId<'_>> = ids
                .iter()
                .map(|id| TrackId::from_id_or_uri(id).unwrap())
                .collect();
            spotify
                .current_user_saved_tracks_add(track_ids)
                .await
                .unwrap();
            print_json(&json!({
                "type": "track",
                "saved": ids.len(),
                "ids": ids,
            }));
        }
        ItemType::Album => {
            let album_ids: Vec<AlbumId<'_>> = ids
                .iter()
                .map(|id| AlbumId::from_id_or_uri(id).unwrap())
                .collect();
            spotify
                .current_user_saved_albums_add(album_ids)
                .await
                .unwrap();
            print_json(&json!({
                "type": "album",
                "saved": ids.len(),
                "ids": ids,
            }));
        }
        ItemType::Playlist => {
            for id in ids {
                let playlist_id = PlaylistId::from_id_or_uri(id).unwrap();
                spotify.playlist_follow(playlist_id, None).await.unwrap();
            }
            print_json(&json!({
                "type": "playlist",
                "saved": ids.len(),
                "ids": ids,
            }));
        }
        ItemType::Artist => {
            print_json(&json!({
                "error": "saving artists is not supported; use 'follow' instead"
            }));
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Configure does not require authentication.
    if let Command::Configure = &cli.command {
        cmd_configure();
        return;
    }

    let spotify = authenticate().await;

    match &cli.command {
        Command::Search {
            query,
            r#type,
            limit,
        } => cmd_search(&spotify, query, r#type, *limit).await,
        Command::Playlist { command } => match command {
            PlaylistCommand::List => cmd_playlist_list(&spotify).await,
            PlaylistCommand::Create {
                name,
                public,
                description,
            } => cmd_playlist_create(&spotify, name, *public, description.as_deref()).await,
            PlaylistCommand::Info { playlist } => cmd_playlist_info(&spotify, playlist).await,
            PlaylistCommand::Add { playlist, tracks } => {
                cmd_playlist_add(&spotify, playlist, tracks).await
            }
        },
        Command::Configure => unreachable!(),
        Command::Save { r#type, ids } => cmd_save(&spotify, r#type, ids).await,
    }
}
