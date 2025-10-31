use std::fs;

use std::path::PathBuf;
use std::fs::File;
use toml::{Table, Value};
use toml::map::Map;
use anyhow::{Context, Result};

use crate::profile::{Profile, LOCAL_K, REMOTE_K};

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
    pub fn get_profiles(&mut self, name: &String) -> Result<Vec<Profile>> {
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
    pub fn get_leaves_profiles(&self) -> Result<Vec<Profile>> {
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
        name: String,
        profile: Table) -> Result<()> {
        self.toml
            .insert(
                name,
                toml::Value::Table(profile));
        Ok(())
    }

    pub fn add_sub_profile(
        &mut self,
        name_parent: &str,
        name: String,
        profile: Table) -> Result<()> {
        if !self.toml.contains_key(name_parent) {
            self.toml.insert(
                name_parent.to_string(),
                toml::Value::Table(Table::new()));
        }

        let profile_root = get_sub_table(&mut self.toml, name_parent)?;
        if table_is_leaf(profile_root) {
            anyhow::bail!(format!("Profile {} is leaf", name_parent));
        };
        profile_root
            .insert(
                name,
                toml::Value::Table(profile));
        Ok(())
    }

    pub fn remove_root_profile(
        &mut self,
        profile: &String) -> Result<Vec<Profile>> {
        let profiles = self.get_profiles(profile)?;
        self.toml.remove(profile).context(format!("Error removing root profile {}", profile))?;
        Ok(profiles)
    }

    pub fn remove_sub_profile(
        &mut self,
        root_profile: &str,
        sub_profile: &str) -> Result<Vec<Profile>> {
        let profile_root = get_sub_table(&mut self.toml, root_profile)?;
        if table_is_leaf(profile_root) {
            anyhow::bail!(format!("Profile {} is leaf", root_profile));
        };
        Ok(vec![Profile::from_table(
                format!("{}.{}", root_profile, sub_profile),
                profile_root
                    .remove(sub_profile)
                    .context(format!("Subprofile {} not found", sub_profile))?
                    .as_table()
                    .context(format!("Subprofile {} is not a table", sub_profile))?)?])
    }
}

pub fn get_path(override_path: &Option<PathBuf>) -> Result<PathBuf> {
    let config_path = match override_path.clone() {
        Some(path) => path,
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
