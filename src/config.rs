use std::fs;

use std::path::PathBuf;
use std::fs::File;
use toml::{Table, Value};
use toml::map::Map;
use anyhow::{Context, Result};

use crate::profile::{Profile, ProfileName, LOCAL_K, REMOTE_K};

const CONFIG_DIR: &str = "ez-sync";
const CONFIG_FILE_PROFILES: &str = "profiles.toml";

pub struct Config {
    toml: Table,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let str = std::fs::read_to_string(path)?;
        let toml: Table = str.parse()?;

        Ok(Config { toml })
    }

    pub fn save(&self, path: PathBuf) -> Result<()> {
        let str = toml::to_string_pretty(&self.toml)?;
        std::fs::write(path, str)?;

        Ok(())
    }

    /// Returns a vector with a single profile if its childless or
    /// with its sub profiles otherwise
    pub fn get_profiles(&self, profile_name: &ProfileName) -> Result<Vec<Profile>> {
        match profile_name {
            ProfileName::Root(name) => {
                let profile_root = get_sub_table(&self.toml, name)?;
                if table_is_leaf(profile_root) {
                return Ok(vec![Profile::from_table(
                    profile_name.clone(),
                    profile_root)?]);
                };
                
                Ok(profile_root
                    .iter()
                    .filter_map(|(_, value_sub)| match value_sub {
                        Value::Table (map_sub) => {
                            Some(Profile::from_table(
                                    profile_name.clone(),
                                    map_sub).ok()?)
                        },
                        _ => None,
                    }).collect::<Vec<Profile>>())
            }
            ProfileName::Child(parent_name, name) => {
                let profile_root = get_sub_table(&self.toml, parent_name)?;
                Ok(vec![Profile::from_table(
                    profile_name.clone(),
                    get_sub_table(
                        profile_root,
                        name)?)?])
            }
        }
    }

    /// Returns sub profiles and childless profiles
    pub fn get_leaves_profiles(&self) -> Result<Vec<Profile>> {
        let profiles_iter = self.toml
            .iter()
            .filter_map(|(name, value)|
                match value {
                    Value::Table (map) => {
                        let name = ProfileName::from(name).ok()?;
                        Some(Profile::from_table(name, map).ok()?)
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
                                        let name = format!(
                                            "{}.{}",
                                            name_root.clone(),
                                            name_sub.clone());
                                        let name = ProfileName::from(&name).ok()?;
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

    pub fn add_profile(
        &mut self,
        name: ProfileName,
        profile: Table) -> Result<()> {
        match name {
            ProfileName::Root(name) => {
                self.toml.insert(name, toml::Value::Table(profile));
            }
            ProfileName::Child(parent_name, name) => {
                if !self.toml.contains_key(&parent_name) {
                    self.toml.insert(
                        parent_name.clone(),
                        toml::Value::Table(Table::new()));
                };

                let profile_root = get_sub_table_mut(&mut self.toml, &parent_name)?;
                if table_is_leaf(profile_root) {
                    anyhow::bail!(format!("Profile {} is leaf", parent_name));
                };
                profile_root.insert(name, toml::Value::Table(profile));
            }
        };

        Ok(())
    }

    pub fn remove_profile(
        &mut self,
        profile_name: ProfileName) -> Result<Vec<Profile>> {
        match &profile_name {
            ProfileName::Root(name) => {
                let profiles = self.get_profiles(&profile_name)?;
                self.toml.
                    remove(name).
                    context(format!("Error removing root profile {}", name))?;
                Ok(profiles)
            }
            ProfileName::Child(parent_name, name) => {
                let profile_root = get_sub_table_mut(&mut self.toml, parent_name)?;
                if table_is_leaf(profile_root) {
                    anyhow::bail!(format!("Profile {} is leaf", parent_name));
                };
                Ok(vec![Profile::from_table(
                    profile_name.clone(),
                    profile_root
                        .remove(name)
                        .context(format!("Subprofile {} not found", name))?
                        .as_table()
                        .context(format!("Subprofile {} is not a table", name))?)?])
            }
        }
    }
}

pub fn get_path(override_path: &Option<PathBuf>) -> Result<PathBuf> {
    let config_path = match override_path.clone() {
        Some(path) => {
            if !path.exists() {
                std::fs::File::create(&path)?;
            };
            path
        }
        None => {
            let mut config = dirs::config_dir().context(
                "Config dir not found, specify with --config <config>")?;

            config.push(CONFIG_DIR);

            fs::create_dir_all(&config).context(
                format!(
                    "Failed to create config folder at {}",
                    &config.display()))?;

            config.push(CONFIG_FILE_PROFILES);
            config
        },
    };

    if !config_path.exists() {
        if override_path.is_none() {
            File::create(&config_path)?;
        } else {
            anyhow::bail!("Specified config file not found");
        }
    };

    Ok(config_path)
}

fn get_sub_table<'a>(
    map: &'a Map<String, Value>,
    name: &str) -> Result<&'a Map<String, Value>> {
    map
        .get(name)
        .context(format!("Child {} not found", name))?
        .as_table()
        .context(format!("Child {} not a table", name))
}

fn get_sub_table_mut<'a>(
    map: &'a mut Map<String, Value>,
    name: &str) -> Result<&'a mut Map<String, Value>> {
    map
        .get_mut(name)
        .context(format!("Child {} not found", name))?
        .as_table_mut()
        .context(format!("Child {} not a table", name))
}

fn table_is_leaf(table: &Map<String, Value>) -> bool {
        table.contains_key(LOCAL_K) &&
        table.contains_key(REMOTE_K)
}
