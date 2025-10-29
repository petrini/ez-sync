use crate::profile;
use crate::config;

use std::path::PathBuf;
use std::error::Error;
use clap::{Parser, Subcommand};

const ALL_PROFILES: &str = "all";

#[derive(Subcommand)]
enum Command {
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
    command: Option<Command>,
    /// Uses specified config instead of default one
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub fn validate_args() -> Result<profile::Command, Box<dyn Error>> {
    let args = Args::parse();

    let Some(command) = args.command else { 
        return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Missing command, use --help for command list")));
    };

    let config = config::Config::from(&args.config)?;

    let command = match command {
        Command::Add { name, local, remote } => {
            if name == ALL_PROFILES {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Cannot use '{}' as profile name", ALL_PROFILES))));
            }

            profile::Command::Add(
                config,
                profile::Profile::new(
                    name, local, remote))
        }
        Command::Remove { name } => {
            if name == ALL_PROFILES {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot delete all profiles. Manually delete the config file instead")));
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
