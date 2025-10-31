use std::path::PathBuf;

use crate::config::Config;
use colored::Colorize;
use anyhow::{Context, Result};
use toml::{Table, Value};

pub const LOCAL_K: &str = "local";
pub const REMOTE_K: &str = "remote";

pub struct Profile {
    pub name: String,
    pub local: PathBuf,
    pub remote: PathBuf,
}

pub struct ProfileSync {
    pub name: String,
    pub source: PathBuf,
    pub target: PathBuf,
}

pub enum Command {
    Add(Config, String, Table),
    Remove(Config, String),
    Sync(Vec<ProfileSync>),
    List(Vec<Profile>),
}

impl Profile {
    pub fn from_table(name: String, table: &Table) -> Result<Self> {
        let local = PathBuf::from(shellexpand::env(table
            .get(LOCAL_K)
            .context(format!("Key {} not found", LOCAL_K))?
            .as_str()
            .context(format!("Key {} not a string", LOCAL_K))?)?
            .to_string());
        let remote = PathBuf::from(shellexpand::env(table
            .get(REMOTE_K)
            .context(format!("Key {} not found", LOCAL_K))?
            .as_str()
            .context(format!("Key {} not a string", LOCAL_K))?)?
            .to_string());
        Ok(Profile { name, local, remote })
    }

    pub fn push(self) -> ProfileSync {
        ProfileSync {
            name: self.name,
            source: self.local,
            target: self.remote,
        }
    }

    pub fn pull(self) -> ProfileSync {
        ProfileSync {
            name: self.name,
            source: self.remote,
            target: self.local,
        }
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "[{}] {}: {} {}: {}",
            self.name.green().bold(),
            LOCAL_K.yellow(),
            self.local.display(),
            REMOTE_K.yellow(),
            self.remote.display())
    }
}

pub fn create_profile_table(local: String, remote: String) -> Result<Table> {
    let mut table = Table::with_capacity(2);
    table.insert(
        LOCAL_K.to_string(),
        Value::String(local));
    table.insert(
        REMOTE_K.to_string(),
        Value::String(remote));

    Ok(table)
}
