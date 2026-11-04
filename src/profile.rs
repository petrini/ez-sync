use std::path::PathBuf;

use crate::config::Config;
use anyhow::{Context, Result};
use colored::Colorize;
use toml::{Table, Value};

pub const LOCAL_K: &str = "local";
pub const REMOTE_K: &str = "remote";

pub struct Profile {
    pub name: ProfileName,
    pub local: PathBuf,
    pub remote: PathBuf,
}

pub struct ProfileSync {
    pub name: ProfileName,
    pub source: PathBuf,
    pub target: PathBuf,
}

pub enum Command {
    Add(Config, String, Table),
    Remove(Config, String),
    Sync(Vec<ProfileSync>),
    List(Vec<Profile>),
}

#[derive(Clone, Debug)]
pub enum ProfileName {
    Root(String),
    Child(String, String),
}

impl Profile {
    pub fn from_table(name: ProfileName, table: &Table) -> Result<Self> {
        let local = PathBuf::from(
            shellexpand::env(
                table
                    .get(LOCAL_K)
                    .context(format!("Key {} not found", LOCAL_K))?
                    .as_str()
                    .context(format!("Key {} not a string", LOCAL_K))?,
            )?
            .to_string(),
        );
        let remote = PathBuf::from(
            shellexpand::env(
                table
                    .get(REMOTE_K)
                    .context(format!("Key {} not found", LOCAL_K))?
                    .as_str()
                    .context(format!("Key {} not a string", LOCAL_K))?,
            )?
            .to_string(),
        );
        Ok(Profile {
            name,
            local,
            remote,
        })
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

impl ProfileName {
    pub fn from(full_name: &str) -> Result<Self> {
        let profile_split: Vec<_> = full_name.split(".").collect();
        match profile_split.len() {
            2 => Ok(ProfileName::Child(
                profile_split[0].to_owned(),
                profile_split[1].to_owned(),
            )),
            1 => Ok(ProfileName::Root(profile_split[0].to_owned())),
            _ => anyhow::bail!(format!(
                "Only single and double layered profiles supported, {} is invalid",
                full_name
            )),
        }
    }
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {} {}: {}",
            self.name,
            LOCAL_K.yellow(),
            self.local.display(),
            REMOTE_K.yellow(),
            self.remote.display()
        )
    }
}

impl std::fmt::Display for ProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProfileName::Root(name) => write!(f, "{}", name.green().bold()),
            ProfileName::Child(parent_name, name) => {
                write!(f, "{}.{}", parent_name.green().bold(), name.green().bold())
            }
        }
    }
}

pub fn create_profile_table(local: String, remote: String) -> Result<Table> {
    let mut table = Table::with_capacity(2);
    table.insert(LOCAL_K.to_string(), Value::String(local));
    table.insert(REMOTE_K.to_string(), Value::String(remote));

    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(local: &str, remote: &str) -> Table {
        create_profile_table(local.to_string(), remote.to_string()).unwrap()
    }

    #[test]
    fn name_root_and_child() {
        assert!(matches!(ProfileName::from("a").unwrap(), ProfileName::Root(n) if n == "a"));
        assert!(
            matches!(ProfileName::from("a.b").unwrap(), ProfileName::Child(p, c) if p == "a" && c == "b")
        );
    }

    #[test]
    fn name_rejects_three_levels() {
        assert!(ProfileName::from("a.b.c").is_err());
    }

    #[test]
    fn from_table_reads_paths() {
        let p = Profile::from_table(ProfileName::from("x").unwrap(), &table("/l/", "/r")).unwrap();
        assert_eq!(p.local, PathBuf::from("/l/"));
        assert_eq!(p.remote, PathBuf::from("/r"));
    }

    #[test]
    fn from_table_expands_env() {
        // PATH is always set, so expansion must succeed and drop the `$`
        let p = Profile::from_table(ProfileName::from("x").unwrap(), &table("$PATH/l", "/r"))
            .unwrap();
        assert!(!p.local.to_string_lossy().contains('$'));
    }

    #[test]
    fn from_table_fails_on_unset_env_var() {
        let t = table("$EZ_SYNC_SURELY_UNSET_VAR/l", "/r");
        assert!(Profile::from_table(ProfileName::from("x").unwrap(), &t).is_err());
    }

    #[test]
    fn from_table_requires_both_keys() {
        let mut t = Table::new();
        t.insert(LOCAL_K.to_string(), Value::String("/l".into()));
        assert!(Profile::from_table(ProfileName::from("x").unwrap(), &t).is_err());
    }

    #[test]
    fn push_goes_local_to_remote() {
        let p = Profile::from_table(ProfileName::from("x").unwrap(), &table("/l", "/r")).unwrap();
        let s = p.push();
        assert_eq!(s.source, PathBuf::from("/l"));
        assert_eq!(s.target, PathBuf::from("/r"));
    }

    #[test]
    fn pull_goes_remote_to_local() {
        let p = Profile::from_table(ProfileName::from("x").unwrap(), &table("/l", "/r")).unwrap();
        let s = p.pull();
        assert_eq!(s.source, PathBuf::from("/r"));
        assert_eq!(s.target, PathBuf::from("/l"));
    }
}
