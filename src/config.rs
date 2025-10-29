use std::fs;

use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use toml::{Table, Value};

use crate::profile::Profile;

const CONFIG_DIR: &str = "ez-sync";
const CONFIG_FILE_PROFILES: &str = "profiles.toml";

pub struct Config {
    toml: Table,
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

    /// Returns a vector with a single profile if its childless or
    /// with its sub profiles otherwise
    pub fn get_profiles(&self, name: &String) -> Result<Vec<Profile>, std::io::Error> {
        let profile_root = self.toml
            .get(name)
            .ok_or(std::io::ErrorKind::NotFound)?
            .as_table()
            .ok_or(std::io::ErrorKind::NotFound)?;
        if profile_root.contains_key("local") && profile_root.contains_key("remote") {
            return Ok(vec![ Profile::from(name.to_string(), profile_root)? ]);
        }

        let sub_profiles = profile_root
            .iter()
            .filter_map(|(name_sub, value_sub)| match value_sub {
                    Value::Table (map_sub) => {
                        let name = format!("{}.{}",name.clone(), name_sub.clone());
                        Some(Profile::from(name, map_sub).ok()?)
                    },
                    _ => None,
            }).collect::<Vec<Profile>>();

        Ok(sub_profiles)
    }

    /// Returns sub profiles and childless profiles
    pub fn get_leaves_profiles(&self) -> Result<Vec<Profile>, std::io::Error> {
        let profiles_iter = self.toml
            .iter()
            .filter_map(|(name, value)|
                match value {
                    Value::Table (map) => {
                        Some(Profile::from(name.clone(), map).ok()?)
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
                                        Some(Profile::from(name, map_sub).ok()?)
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
