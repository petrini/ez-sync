mod input;
mod config;
mod profile;

use tokio::process;

use std::time::Duration;
use std::sync::Arc;
use profile::{Profile, ProfileSync};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use colored::Colorize;
use anyhow::{Error, Result};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = input::parse_args();
    let config_path = config::get_path(&args.config)?;
    let config = config::Config::load(&config_path)?;

    match input::validate_command(config, args.command)? {
        profile::Command::Add(mut config, profile_name, profile_table) => {
            let name = profile::ProfileName::from(&profile_name)?;
            config.add_profile(name, profile_table)?;
            config.save(config_path)?;
            println!("Profile added");
            println!("{}", profile_name);
            Ok(())
        },
        profile::Command::Remove(mut config, profile) => {
            let name = profile::ProfileName::from(&profile)?;
            let removed_profiles = config.remove_profile(name)?;
            config.save(config_path)?;
            println!("Profile/s removed");
            list_profiles(&removed_profiles);
            Ok(())
        },
        profile::Command::Sync(profile_syncs) => {
            let mp_arc = Arc::new(MultiProgress::new());
            let mut join_set = tokio::task::JoinSet::new();
            for ps in profile_syncs {
                join_set.spawn(execute_rsync(ps, Arc::clone(&mp_arc)));
            }
            join_set.join_all().await;
            Ok(())
        },
        profile::Command::List (profiles) => {
            list_profiles(&profiles);
            Ok(())
        },
    }
}

fn list_profiles(profiles: &Vec<Profile>) {
    for profile in profiles {
        println!("{}", profile);
    }
}

async fn execute_rsync(profile_sync: ProfileSync, multi_progress: Arc<MultiProgress>) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_draw_target(ProgressDrawTarget::hidden());
    spinner.set_prefix(format!("[{}]:", profile_sync.name.to_string().green().bold()));
    spinner.set_style(
        ProgressStyle::default_spinner()
        .template("{prefix} {spinner}{msg}")
        .expect("Failed to create spinner template"));
    multi_progress.add(spinner.clone());
    spinner.enable_steady_tick(Duration::from_millis(100));

    let result = process::Command::new("rsync")
        .arg("-a")
        .arg("--delete")
        .arg(format!("{}", profile_sync.source.display()))
        .arg(format!("{}", profile_sync.target.display()))
        .output()
        .await;

    spinner.finish();
    match result {
       Ok(_) => spinner.set_message("done"),
       Err(_) => spinner.set_message("failed"),
    };

    Ok(())
}
