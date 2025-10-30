use crate::config::Config;
use std::path::PathBuf;
use colored::Colorize;

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
    Add(Config, Profile),
    Remove(Config, String),
    Sync(Vec<ProfileSync>),
    List(Vec<Profile>),
}

impl Profile {
    pub fn from_table(name: String, table: &toml::Table) -> Result<Self, std::io::Error> {
        let local = PathBuf::from(table
            .get(LOCAL_K)
            .ok_or(std::io::ErrorKind::NotFound)?
            .as_str()
            .ok_or(std::io::ErrorKind::NotFound)?);
        let remote = PathBuf::from(table
            .get(REMOTE_K)
            .ok_or(std::io::ErrorKind::NotFound)?
            .as_str()
            .ok_or(std::io::ErrorKind::NotFound)?);
        Ok(Profile { name, local, remote })
    }

    pub fn to_table(&self) -> Result<toml::Table, std::io::Error> {
        let mut table = toml::Table::with_capacity(2);
        table.insert(
            LOCAL_K.to_string(),
            toml::Value::String(
                self.local
                .display()
                .to_string()));
        table.insert(
            REMOTE_K.to_string(),
            toml::Value::String(
                self.remote
                .display()
                .to_string()));
        Ok(table)
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
