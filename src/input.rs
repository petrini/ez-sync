use crate::processing;

use std::path::PathBuf;
use std::error::Error;
use std::collections::HashMap;
use clap::{Parser, Subcommand};

const ALL_PROFILES: &str = "all";

#[derive(Subcommand)]
enum Command {
    /// Adds a profile, use add <name> <local-dir> <remote-dir> (optional)--rsync-params <params>
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
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// Pulls a profile from target to local, use pull <name> (optional)--force
    Pull {
        name: String,
        #[arg(long, default_value_t = false)]
        force: bool,
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

pub fn validate_args() -> Result<processing::Command, Box<dyn Error>> {
    let args = Args::parse();

    let Some(command) = args.command else { 
        return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Missing command, use --help for command list")));
    };

    let config = processing::Config::from(&args.config)?;

    let command = match command {
        Command::Add { name, local, remote } => {
            if name == ALL_PROFILES {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Cannot use '{}' as profile name", ALL_PROFILES))));
            }

            processing::Command::Add(
                config,
                processing::Profile::new(
                    name, local, remote))
        }
        Command::Remove { name } => {
            if name == ALL_PROFILES {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot delete all profiles. Manually delete the config file instead")));
            }

            processing::Command::Remove(config, name)
        }
        Command::Push { name, force } => {
            let profiles = if name == ALL_PROFILES {
                config.get_leaves_profiles()?
            } else {
                config.get_profiles(&name)?
            };
            let profile_syncs = profiles
                .into_iter()
                .map(|p| p.push())
                .collect();
            if !force {
                validate_dates(&profile_syncs)?;
            };

            processing::Command::Sync(profile_syncs)
        },
        Command::Pull { name, force } => {
            let profiles = if name == ALL_PROFILES {
                config.get_leaves_profiles()?
            } else {
                config.get_profiles(&name)?
            };
            let profile_syncs = profiles
                .into_iter()
                .map(|p| p.pull())
                .collect();
            if !force {
                validate_dates(&profile_syncs)?;
            };

            processing::Command::Sync(profile_syncs)
        },
        Command::List => 
            processing::Command::List(config.get_leaves_profiles()?),
    };

    Ok(command)
}

fn validate_dates(profile_syncs: &Vec<processing::ProfileSync>) -> Result<(), std::io::Error> {
    for profile_sync in profile_syncs {
        let target_files = walkdir::WalkDir::new(&profile_sync.target)
            .into_iter()
            .filter_map(|entry_res| {
                let entry = entry_res.ok()?;
println!("TARGET {:?}", entry.path());
                Some((entry
                        .path()
                        .strip_prefix(&profile_sync.target)
                        .ok()?
                        .to_path_buf(),
                    entry.metadata().ok()?))
            })
            .collect::<HashMap<_, _>>();
        for entry in walkdir::WalkDir::new(&profile_sync.source)
            .into_iter()
            .filter_map(|entry_res| entry_res.ok()) {
                let source_meta = entry.metadata()?;
                let source_path = entry
                    .path()
                    .strip_prefix(&profile_sync.source)
                    .map_err(|_|
                        std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("Error reading path {}", entry.path().display())))?;
                if let Some(target_meta) = target_files.get(&source_path.to_path_buf()) {
                    let source_mod = source_meta.modified()?;
                    let target_mod = target_meta.modified()?;
                    if source_mod < target_mod {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            format!("Source {} is older than target file, use --force",
                                entry.path().display())));
                        }
                }
        }
    }

    Ok(())
}
