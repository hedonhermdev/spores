# AGENTS.md

This file provides guidance for AI coding agents working in the `spores` repository.

## Project Overview

Spores is a Rust CLI tool for Spotify playlist management via the Spotify Web API. It is a single-crate binary with all source code in `src/main.rs`. It uses `rspotify` for API interaction, `clap` (derive) for CLI parsing, and outputs structured JSON.

- **Language:** Rust (edition 2024)
- **Async runtime:** Tokio (full features)
- **Key crates:** clap 4, rspotify 0.15.3, serde/serde_json, toml, dirs

## Build / Lint / Test Commands

```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run the CLI during development
cargo run -- <subcommand> [args]

# Format code (uses default rustfmt settings)
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Lint with clippy
cargo clippy -- -D warnings

# Run all tests
cargo test

# Run a single test by name
cargo test <test_name>

# Run tests in a specific module
cargo test <module_path>::

# Run tests with output shown
cargo test -- --nocapture

# Type-check without building
cargo check
```

There are currently no tests in the project. When adding tests, use standard Rust `#[test]` or `#[tokio::test]` attributes. Place unit tests in a `#[cfg(test)] mod tests` block at the bottom of `src/main.rs`, or create a `tests/` directory for integration tests.

There is no CI/CD pipeline, no Makefile, no custom build scripts, and no pre-commit hooks. No `rustfmt.toml`, `clippy.toml`, or `rust-toolchain.toml` exists -- the project uses Rust defaults for all tooling.

## Code Style Guidelines

### Formatting

- Use `cargo fmt` defaults (no custom rustfmt config).
- 4-space indentation (Rust default).
- No trailing whitespace.

### File Organization

The codebase uses horizontal-rule comment banners to separate logical sections:

```rust
// ---------------------------------------------------------------------------
// Section Name
// ---------------------------------------------------------------------------
```

Current sections in order: Config, CLI, Helpers, Auth, Search, Playlist commands, Save to library, Main. Follow this convention when adding new sections.

### Imports

- Group imports with `std` first, then external crates, then local modules (if any).
- Use nested imports from the same crate (e.g., `use rspotify::{model::{...}, prelude::*, ...}`).
- Import traits via `use rspotify::prelude::*` to bring Spotify API methods into scope.
- Prefer importing specific items over glob imports (except for `prelude::*`).

### Types and Structs

- Derive `Deserialize` for config structs (via `serde`).
- Derive `Parser`/`Subcommand`/`ValueEnum` for CLI types (via `clap`).
- Use `#[command(...)]` and `#[arg(...)]` attributes for clap metadata.
- Prefer `Option<T>` for optional CLI arguments and config fields.

### Naming Conventions

- Standard Rust naming: `snake_case` for functions/variables, `CamelCase` for types/enums.
- Command handler functions are named `cmd_<command>` (e.g., `cmd_search`, `cmd_playlist_list`).
- Prefix async handler functions with `cmd_`.
- Use `_str` suffix for string representations of IDs that will be parsed (e.g., `playlist_id_str`).

### Error Handling

- The codebase currently uses `.unwrap()` liberally on Results. This is the existing pattern.
- For config/startup errors, use `eprintln!()` followed by `process::exit(1)`.
- For panics with context, use `.unwrap_or_else(|e| panic!("message: {e}"))`.
- When adding new code, follow the existing `.unwrap()` pattern for consistency, unless adding proper error handling is specifically requested.

### Async Patterns

- All command handlers are `async fn` taking `&AuthCodeSpotify` as the first parameter.
- The `#[tokio::main]` entry point handles CLI parsing, authentication, and dispatch.
- Use `.await` on all rspotify API calls (they are async).

### JSON Output Convention

- All command output goes through `print_json(&Value)` which pretty-prints JSON.
- Build JSON using `serde_json::json!({...})` macro.
- Collect API results into `Vec<Value>` before wrapping in the final JSON structure.
- Include metadata fields like `total`, `query`, `type` alongside `items` arrays.
- User-facing errors should also be JSON: `json!({ "error": "message" })`.

### Adding New Commands

1. **Top-level command:** Add a variant to the `Command` enum, write an `async fn cmd_<name>` handler, add a match arm in `main()`.
2. **Playlist subcommand:** Add a variant to `PlaylistCommand`, write the handler, add a match arm in the `Command::Playlist` branch.

### rspotify API Patterns

- Use `PlaylistId::from_id_or_uri()`, `TrackId::from_id_or_uri()`, and `AlbumId::from_id_or_uri()` to accept both raw IDs and Spotify URIs.
- `PlayableItem` enum has `Track`, `Episode`, and unknown variants -- always include a wildcard arm.
- For paginated endpoints, loop with offset/limit and check `.next.is_none()` to stop.
- `playlist_add_items` requires converting `TrackId` -> `PlayableId::Track(id)`.
- `current_user_saved_tracks_add` accepts an iterator of `TrackId` to save tracks to the user's library.
- `current_user_saved_albums_add` accepts an iterator of `AlbumId` to save albums to the user's library.
- `playlist_follow` accepts a `PlaylistId` and optional `public` flag to save (follow) a playlist.

### Configuration

- Config lives at `$XDG_CONFIG_HOME/spores/config.toml` (macOS: `~/Library/Application Support/spores/config.toml`).
- Token cache at `$XDG_CONFIG_HOME/spores/token_cache.json`.
- The redirect URI must use `127.0.0.1` (not `localhost`) -- Spotify rejects localhost.
- Required OAuth scopes: `playlist-read-private`, `playlist-read-collaborative`, `playlist-modify-public`, `playlist-modify-private`, `user-library-modify`.

### Dependencies

Keep dependencies minimal. Current dependency versions are pinned in `Cargo.toml` with major version only (e.g., `clap = "4"`). Do not add new dependencies without clear justification. The `Cargo.lock` is committed and should remain so for reproducible builds.

## Repository Structure

```
.
├── Cargo.toml          # Crate manifest
├── Cargo.lock          # Lockfile (committed)
├── src/
│   └── main.rs         # Entire application
├── skills/
│   └── spores/
│       └── SKILL.md    # AI skill definition (detailed dev guide)
└── README.md           # User-facing documentation
```

For deeper architecture details and rspotify API method reference, see `skills/spores/SKILL.md`.
