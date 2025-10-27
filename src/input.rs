use crate::processing;

use std::path::PathBuf;
use std::error::Error;
use clap::{Parser, Subcommand};

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
            if name == "all" {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot use 'all' as profile name")));
            }

            processing::Command::Add(
                config,
                processing::Profile::new(
                    name, local, remote))
        }
        Command::Remove { name } => processing::Command::Remove(config, name),
        Command::Push { name, force } =>
            processing::Command::List(config.get_leaves_profiles()?),
        Command::Pull { name, force } =>
            processing::Command::List(config.get_leaves_profiles()?),
        Command::List => 
            processing::Command::List(config.get_leaves_profiles()?),
    };

    Ok(command)
}
