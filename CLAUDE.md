# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ez-sync** is a Rust CLI tool that manages named sync profiles for pushing/pulling files between local and remote directories using `rsync` as the backend.

## Common Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo check                # Type-check without building
cargo fmt                  # Format code
cargo clippy               # Lint
cargo run -- <args>        # Run directly
```

There are currently no automated tests.

## Architecture

The project has four modules:

- **`main.rs`** — Entry point. Parses the resolved command, then executes rsync subprocesses concurrently using `tokio::task::JoinSet`. Progress is shown via `indicatif` spinners sharing an `Arc<MultiProgress>`.

- **`input.rs`** — CLI argument parsing via `clap` derive macros. Produces a `Command` enum (Add/Remove/Push/Pull/List). Profile names use dot-notation for hierarchy (`parent.child`); the special name `all` targets every leaf profile.

- **`profile.rs`** — Data models. `ProfileName` is either `Root(String)` or `Child(parent, child)`. `Profile` holds a name + local/remote paths. `ProfileSync` holds source/target paths for a single rsync operation. Push and Pull are implemented as transformations on `ProfileSync`.

- **`config.rs`** — Reads/writes `~/.config/ez-sync/profiles.toml` (XDG via `dirs`). Handles profile CRUD, path expansion via `shellexpand`, and auto-creates the config directory. A profile is a **leaf** (syncable) if its TOML table contains both `local` and `remote` keys; otherwise it is a **parent** that groups children.

**Data flow:** CLI args → `input.rs` → `config.rs` loads profiles → `profile.rs` resolves to `ProfileSync` list → `main.rs` spawns rsync tasks → progress updated → config saved if mutated.

## Key Behaviors

- Rsync is invoked with `-a --delete` (archive mode, remove files absent from source).
- Paths support environment variable expansion (e.g. `$HOME/...`).
- Profile hierarchy is exactly one level deep: root profiles and their direct children.
- `all` as the profile argument syncs every leaf profile, even across different parents.
