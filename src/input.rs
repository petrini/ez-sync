use crate::profile;
use crate::config;

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

const ALL_PROFILES: &str = "all";

#[derive(Subcommand)]
pub enum Command {
    /// Adds a profile, use add <name> <local-dir> <remote-dir>
    Add {
        name: String,
        local: PathBuf,
        remote: PathBuf,
    },
    /// Removes a profile, use remove <name>
    Remove {
        name: String,
    },
    /// Pushes a profile from local to remote, use push <name> (optional)--force
    Push {
        name: String,
    },
    /// Pulls a profile from target to local, use pull <name>
    Pull {
        name: String,
    },
    /// Lists all profiles
    List,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
    /// Uses specified config instead of default one
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub fn parse_args() -> Args {
    Args::parse()
}

pub fn validate_command(mut config: config::Config, command: Option<Command>) -> Result<profile::Command> {
    let Some(command) = command else { 
        anyhow::bail!("Missing command, use --help for command list");
    };

    let command = match command {
        Command::Add { name, local, remote } => {
            if name == ALL_PROFILES {
                anyhow::bail!(format!("Cannot use '{}' as profile name", ALL_PROFILES));
            }

            profile::Command::Add(
                config,
                profile::Profile { name, local, remote })
        }
        Command::Remove { name } => {
            if name == ALL_PROFILES {
                anyhow::bail!("Cannot delete all profiles. Manually delete the config file instead");
            }

            profile::Command::Remove(config, name)
        }
        Command::Push { name } => {
            let profiles = if name == ALL_PROFILES {
                config.get_leaves_profiles()?
            } else {
                config.get_profiles(&name)?
            };
            let profile_syncs = profiles
                .into_iter()
                .map(|p| p.push())
                .collect();

            profile::Command::Sync(profile_syncs)
        },
        Command::Pull { name } => {
            let profiles = if name == ALL_PROFILES {
                config.get_leaves_profiles()?
            } else {
                config.get_profiles(&name)?
            };
            let profile_syncs = profiles
                .into_iter()
                .map(|p| p.pull())
                .collect();

            profile::Command::Sync(profile_syncs)
        },
        Command::List => 
            profile::Command::List(config.get_leaves_profiles()?),
    };

    Ok(command)
}
