use std::fs;

use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use toml::{Table, Value};
use colored::Colorize;

const CONFIG_DIR: &str = "ez-sync";
const CONFIG_FILE_PROFILES: &str = "profiles.toml";

pub struct Config {
    toml: Table,
}

pub struct Profile {
    name: String,
    local: PathBuf,
    remote: PathBuf,
}

pub struct ProfileSync {
    source: PathBuf,
    target: PathBuf,
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
}

impl Config {
    pub fn from(path: &Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let config_path = match path.clone() {
            Some(path) => path,
            None => {
                let mut config = dirs::config_dir().ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Config dir not found, specify with --config <config>")
                })?;

                config.push(CONFIG_DIR);

                fs::create_dir_all(&config).map_err(|_| {
                    std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("Failed to create config folder at {}", &config.display()))
                })?;

                config.push(CONFIG_FILE_PROFILES);
                config
            },
        };

        if !config_path.exists() {
            if path.is_none() {
                File::create(&config_path)?;
            } else {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            "Specified config file not found")));
            }
        };

        let str = std::fs::read_to_string(&config_path)?;
        let toml: Table = str.parse()?;

        Ok(Config { toml })
    }

    pub fn get_leaves_profiles(&self) -> Result<Vec<Profile>, std::io::Error> {
        let profiles_iter = self.toml
            .iter()
            .filter_map(|(name, value)|
                match value {
                    Value::Table (map) => {
                        let name = name.clone();
                        let local = PathBuf::from(map.get("local")?.as_str()?);
                        let remote = PathBuf::from(map.get("remote")?.as_str()?);
                        let profile = Profile { name, local, remote };
                        Some(profile)
                    },
                    _ => None,
                });
        let sub_profiles_iter = self.toml
            .iter()
            .filter_map(|(name_root, value_root)|
                match value_root {
                    Value::Table (map_root) => {
                        Some(map_root
                            .iter()
                            .filter_map(|(name_sub, value_sub)|
                                match value_sub {
                                    Value::Table (map_sub) => {
                                        let name = format!("{}.{}",name_root.clone(), name_sub.clone());
                                        let local = PathBuf::from(map_sub.get("local")?.as_str()?);
                                        let remote = PathBuf::from(map_sub.get("remote")?.as_str()?);
                                        let profile = Profile { name, local, remote };
                                        Some(profile)
                                    },
                                    _ => None,
                                })
                            .collect::<Vec<_>>())
                    },
                    _ => None,
                })
            .flatten();
        Ok(profiles_iter.chain(sub_profiles_iter).collect())
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
