use crate::config::Config;
use std::path::PathBuf;
use toml::Value;
use colored::Colorize;

pub struct Profile {
    name: String,
    local: PathBuf,
    remote: PathBuf,
}

pub struct ProfileSync {
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
    pub fn new(name: String, local: PathBuf, remote: PathBuf) -> Self {
        Profile { name, local, remote }
    }

    pub fn from(name: String, map: &toml::map::Map<String, Value>) -> Result<Self, std::io::Error> {
        let local = PathBuf::from(map
            .get("local")
            .ok_or(std::io::ErrorKind::NotFound)?
            .as_str()
            .ok_or(std::io::ErrorKind::NotFound)?);
        let remote = PathBuf::from(map
            .get("remote")
            .ok_or(std::io::ErrorKind::NotFound)?
            .as_str()
            .ok_or(std::io::ErrorKind::NotFound)?);
        Ok(Profile { name, local, remote })
    }

    pub fn push(self) -> ProfileSync {
        ProfileSync {
            source: self.local,
            target: self.remote,
        }
    }

    pub fn pull(self) -> ProfileSync {
        ProfileSync {
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
            "local".yellow(),
            self.local.display(),
            "remote".yellow(),
            self.remote.display())
    }
}
