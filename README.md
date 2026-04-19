# ez-sync

A CLI tool for managing named sync profiles that push and pull files between local and remote directories using `rsync`.

## Requirements

- Rust toolchain
- `rsync` installed and available in `$PATH`

## Installation

```bash
cargo install --path .
```

## Usage

```
ez-sync <COMMAND>

Commands:
  add     Add a profile
  remove  Remove a profile
  push    Push local → remote
  pull    Pull remote → local
  list    List all profiles
```

### Managing profiles

```bash
# Add a profile
ez-sync add <name> <local-dir> <remote-dir>

# Add a child profile (grouped under a parent)
ez-sync add parent.child ~/docs user@host:/docs

# Remove a profile (removes all children if a parent)
ez-sync remove <name>

# List all profiles
ez-sync list
```

### Syncing

```bash
# Push/pull a specific profile
ez-sync push myprofile
ez-sync pull myprofile

# Push/pull a parent and all its children
ez-sync push parent

# Push/pull every profile at once
ez-sync push all
ez-sync pull all
```

Multiple syncs run concurrently, each with its own progress spinner.

Sync uses `rsync -a --delete` — archive mode, removing files from the target that no longer exist in the source.

## Profile names

Profiles are either flat (`myprofile`) or one level deep (`parent.child`). The name `all` is reserved and targets every leaf profile.

Environment variables are expanded in paths (e.g. `$HOME/docs`).

## Configuration

Profiles are stored in `~/.config/ez-sync/profiles.toml` (created automatically). You can point to a different file with `--config <path>`.

Example config:

```toml
[dotfiles]
local = "$HOME/.config"
remote = "user@host:/home/user/.config"

[projects.work]
local = "$HOME/work"
remote = "user@host:/home/user/work"

[projects.personal]
local = "$HOME/personal"
remote = "user@host:/home/user/personal"
```
