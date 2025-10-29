use std::fs;

use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use toml::{Table, Value};
use toml::map::Map;

use crate::profile::{Profile, LOCAL_K, REMOTE_K};

const CONFIG_DIR: &str = "ez-sync";
const CONFIG_FILE_PROFILES: &str = "profiles.toml";

pub struct Config {
    toml: Table,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let str = std::fs::read_to_string(path)?;
        let toml: Table = str.parse()?;

        Ok(Config { toml })
    }

    pub fn save(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let str = toml::to_string_pretty(&self.toml)?;
        std::fs::write(path, str)?;

        Ok(())
    }

    /// Returns a vector with a single profile if its childless or
    /// with its sub profiles otherwise
    pub fn get_profiles(&mut self, name: &String) -> Result<Vec<Profile>, std::io::Error> {
        let profile_root = get_sub_table(&mut self.toml, name)?;
        if table_is_leaf(profile_root) {
            return Ok(vec![ Profile::from_table(name.to_string(), profile_root)? ]);
        }

        let sub_profiles = profile_root
            .iter()
            .filter_map(|(name_sub, value_sub)| match value_sub {
                    Value::Table (map_sub) => {
                        let name = format!("{}.{}",name.clone(), name_sub.clone());
                        Some(Profile::from_table(name, map_sub).ok()?)
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
                        Some(Profile::from_table(name.clone(), map).ok()?)
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
                                        Some(Profile::from_table(name, map_sub).ok()?)
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

    pub fn add_root_profile(
        &mut self,
        profile: &Profile) -> Result<(), std::io::Error> {
        self.toml
            .insert(
                profile.name.clone(),
                toml::Value::Table(
                    profile.to_table()?));
        Ok(())
    }

    pub fn add_sub_profile(
        &mut self,
        name: &str,
        profile: &Profile) -> Result<(), std::io::Error> {
        let profile_root = get_sub_table(&mut self.toml, name)?;
        if table_is_leaf(profile_root) {
            return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Profile {} is leaf", name)));
        };
        profile_root
            .insert(
                profile.name.clone(),
                toml::Value::Table(
                    profile.to_table()?));
        Ok(())
    }

    pub fn remove_root_profile(
        &mut self,
        profile: &String) -> Result<Vec<Profile>, std::io::Error> {
        let profiles = self.get_profiles(profile)?;
        self.toml.remove(profile).ok_or(std::io::ErrorKind::NotFound)?;
        Ok(profiles)
    }

    pub fn remove_sub_profile(
        &mut self,
        root_profile: &str,
        sub_profile: &str) -> Result<Vec<Profile>, std::io::Error> {
        let profile_root = get_sub_table(&mut self.toml, root_profile)?;
        if table_is_leaf(profile_root) {
            return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Profile {} is leaf", root_profile)));
        };
        Ok(vec![Profile::from_table(
                format!("{}.{}", root_profile, sub_profile),
                profile_root
                    .remove(sub_profile)
                    .ok_or(std::io::ErrorKind::NotFound)?
                    .as_table()
                    .ok_or(std::io::ErrorKind::NotFound)?)?])
    }
}

pub fn get_path(override_path: &Option<PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let config_path = match override_path.clone() {
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
        if override_path.is_none() {
            File::create(&config_path)?;
        } else {
            return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "Specified config file not found")));
        }
    };

    Ok(config_path)
}

fn get_sub_table<'a>(
    map: &'a mut Map<String, Value>,
    name: &str) -> Result<&'a mut Map<String, Value>, std::io::Error> {
    Ok(map
        .get_mut(name)
        .ok_or(std::io::ErrorKind::NotFound)?
        .as_table_mut()
        .ok_or(std::io::ErrorKind::NotFound)?)
}

fn table_is_leaf(table: &Map<String, Value>) -> bool {
        table.contains_key(LOCAL_K) &&
        table.contains_key(REMOTE_K)
}
