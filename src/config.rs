use std::fs;

use anyhow::{Context, Result};
use std::fs::File;
use std::path::PathBuf;
use toml::map::Map;
use toml::{Table, Value};

use crate::profile::{LOCAL_K, Profile, ProfileName, REMOTE_K};

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
                        profile_root,
                    )?]);
                };

                Ok(profile_root
                    .iter()
                    .filter_map(|(key_sub, value_sub)| match value_sub {
                        Value::Table(map_sub) => {
                            let name = ProfileName::from(
                                &format!("{}.{}", name, key_sub)).ok()?;
                            Some(Profile::from_table(
                                    name,
                                    map_sub).ok()?)
                        }
                        _ => None,
                    })
                    .collect::<Vec<Profile>>())
            }
            ProfileName::Child(parent_name, name) => {
                let profile_root = get_sub_table(&self.toml, parent_name)?;
                Ok(vec![Profile::from_table(
                    profile_name.clone(),
                    get_sub_table(profile_root, name)?,
                )?])
            }
        }
    }

    /// Returns sub profiles and childless profiles
    pub fn get_leaves_profiles(&self) -> Result<Vec<Profile>> {
        let profiles_iter = self.toml.iter().filter_map(|(name, value)| match value {
            Value::Table(map) => {
                let name = ProfileName::from(name).ok()?;
                Some(Profile::from_table(name, map).ok()?)
            }
            _ => None,
        });
        let sub_profiles_iter = self
            .toml
            .iter()
            .filter_map(|(name_root, value_root)| match value_root {
                Value::Table(map_root) => Some(
                    map_root
                        .iter()
                        .filter_map(|(name_sub, value_sub)| match value_sub {
                            Value::Table(map_sub) => {
                                let name = format!("{}.{}", name_root.clone(), name_sub.clone());
                                let name = ProfileName::from(&name).ok()?;
                                Some(Profile::from_table(name, map_sub).ok()?)
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .flatten();
        Ok(profiles_iter.chain(sub_profiles_iter).collect())
    }

    pub fn add_profile(&mut self, name: ProfileName, profile: Table) -> Result<()> {
        match name {
            ProfileName::Root(name) => {
                self.toml.insert(name, toml::Value::Table(profile));
            }
            ProfileName::Child(parent_name, name) => {
                if !self.toml.contains_key(&parent_name) {
                    self.toml
                        .insert(parent_name.clone(), toml::Value::Table(Table::new()));
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

    pub fn remove_profile(&mut self, profile_name: ProfileName) -> Result<Vec<Profile>> {
        match &profile_name {
            ProfileName::Root(name) => {
                let profiles = self.get_profiles(&profile_name)?;
                self.toml
                    .remove(name)
                    .context(format!("Error removing root profile {}", name))?;
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
                        .context(format!("Subprofile {} is not a table", name))?,
                )?])
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
            let mut config = dirs::config_dir()
                .context("Config dir not found, specify with --config <config>")?;

            config.push(CONFIG_DIR);

            fs::create_dir_all(&config).context(format!(
                "Failed to create config folder at {}",
                &config.display()
            ))?;

            config.push(CONFIG_FILE_PROFILES);
            config
        }
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

fn get_sub_table<'a>(map: &'a Map<String, Value>, name: &str) -> Result<&'a Map<String, Value>> {
    map.get(name)
        .context(format!("Child {} not found", name))?
        .as_table()
        .context(format!("Child {} not a table", name))
}

fn get_sub_table_mut<'a>(
    map: &'a mut Map<String, Value>,
    name: &str,
) -> Result<&'a mut Map<String, Value>> {
    map.get_mut(name)
        .context(format!("Child {} not found", name))?
        .as_table_mut()
        .context(format!("Child {} not a table", name))
}

fn table_is_leaf(table: &Map<String, Value>) -> bool {
    table.contains_key(LOCAL_K) && table.contains_key(REMOTE_K)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::create_profile_table;

    fn name(s: &str) -> ProfileName {
        ProfileName::from(s).unwrap()
    }

    fn add(config: &mut Config, n: &str) {
        let t = create_profile_table(format!("/l/{n}"), format!("/r/{n}")).unwrap();
        config.add_profile(name(n), t).unwrap();
    }

    fn empty() -> Config {
        Config { toml: Table::new() }
    }

    fn names(profiles: &[Profile]) -> Vec<String> {
        let mut v: Vec<_> = profiles
            .iter()
            .map(|p| match &p.name {
                ProfileName::Root(n) => n.clone(),
                ProfileName::Child(a, b) => format!("{a}.{b}"),
            })
            .collect();
        v.sort();
        v
    }

    #[test]
    fn root_profile_roundtrip() {
        let mut c = empty();
        add(&mut c, "docs");
        let got = c.get_profiles(&name("docs")).unwrap();
        assert_eq!(names(&got), ["docs"]);
        assert_eq!(got[0].local, PathBuf::from("/l/docs"));
    }

    #[test]
    fn parent_returns_all_children() {
        let mut c = empty();
        add(&mut c, "p.a");
        add(&mut c, "p.b");
        assert_eq!(names(&c.get_profiles(&name("p")).unwrap()), ["p.a", "p.b"]);
        assert_eq!(names(&c.get_profiles(&name("p.a")).unwrap()), ["p.a"]);
    }

    #[test]
    fn missing_profile_is_error() {
        assert!(empty().get_profiles(&name("nope")).is_err());
    }

    #[test]
    fn cannot_add_child_under_leaf() {
        let mut c = empty();
        add(&mut c, "leaf");
        let t = create_profile_table("/l".into(), "/r".into()).unwrap();
        assert!(c.add_profile(name("leaf.child"), t).is_err());
    }

    #[test]
    fn leaves_span_parents_and_exclude_parents() {
        let mut c = empty();
        add(&mut c, "solo");
        add(&mut c, "p.a");
        add(&mut c, "q.b");
        assert_eq!(
            names(&c.get_leaves_profiles().unwrap()),
            ["p.a", "q.b", "solo"]
        );
    }

    #[test]
    fn remove_root_returns_removed_children() {
        let mut c = empty();
        add(&mut c, "p.a");
        add(&mut c, "p.b");
        let removed = c.remove_profile(name("p")).unwrap();
        assert_eq!(names(&removed), ["p.a", "p.b"]);
        assert!(c.get_profiles(&name("p")).is_err());
    }

    #[test]
    fn remove_child_keeps_siblings() {
        let mut c = empty();
        add(&mut c, "p.a");
        add(&mut c, "p.b");
        c.remove_profile(name("p.a")).unwrap();
        assert_eq!(names(&c.get_leaves_profiles().unwrap()), ["p.b"]);
    }

    #[test]
    fn remove_missing_child_is_error() {
        let mut c = empty();
        add(&mut c, "p.a");
        assert!(c.remove_profile(name("p.zzz")).is_err());
    }

    #[test]
    fn table_is_leaf_needs_both_keys() {
        let mut t = Table::new();
        assert!(!table_is_leaf(&t));
        t.insert(LOCAL_K.into(), Value::String("a".into()));
        assert!(!table_is_leaf(&t));
        t.insert(REMOTE_K.into(), Value::String("b".into()));
        assert!(table_is_leaf(&t));
    }
}
