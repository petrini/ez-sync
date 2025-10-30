mod input;
mod config;
mod profile;

//use std::process;
use std::time::Duration;

use profile::Profile;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle, ProgressDrawTarget};
use colored::Colorize;
use tokio::process;
use futures::future;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = input::parse_args();
    let config_path = config::get_path(&args.config)?;
    let config = config::Config::load(&config_path)?;

    match input::validate_command(config, args.command)? {
        profile::Command::Add(mut config, mut profile) => {
            let full_name = profile.name.clone();
            let profile_split: Vec<_> = profile.name
                .split(".")
                .map(|s| s.to_string())
                .collect();
            match profile_split.len() {
                2 => {
                    profile.name = profile_split[1].clone();
                    config.add_sub_profile(&profile_split[0], &profile)?;
                },
                1 => config.add_root_profile(&profile)?,
                _ => return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Only single and double layered profiles supported, {} is invalid",
                        profile.name)))),
            }?;

            config.save(config_path)?;
            profile.name = full_name;
            println!("Profile added");
            println!("{}", profile);
            Ok(())
        },
        profile::Command::Remove(mut config, profile) => {
            let profile_split: Vec<_> = profile.split(".").collect();
            let removed_profiles = match profile_split.len() {
                2 => config.remove_sub_profile(profile_split[0], profile_split[1])?,
                1 => config.remove_root_profile(&profile)?,
                _ => return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Only single and double layered profiles supported, {} is invalid",
                        profile)))),
            };

            config.save(config_path)?;
            println!("Profile/s removed");
            list_profiles(&removed_profiles);
            Ok(())
        },
        profile::Command::Sync(profile_syncs) => {
            let mp = MultiProgress::new();
            let sync_futures = profile_syncs.into_iter().map(|ps| execute_rsync(ps, &mp)).collect();
            for result in future::join_all(sync_futures) {
                if Err(e) == result {
                    return result;
                }
            }

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

async fn execute_rsync(profile_sync: &ProfileSync, multi_progress: &MultiProgress) -> Result<(), std::io::Error> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_draw_target(ProgressDrawTarget::hidden());
    spinner.set_prefix(format!("[{}]:", profile_sync.name.to_string().green().bold()));
    spinner.set_style(
        ProgressStyle::default_spinner()
        .template("{prefix} {spinner}{msg}")
        .expect("Failed to create spinner template"));
    multi_progress.add(spinner.clone());
    spinner.enable_steady_tick(Duration::from_millis(100));

    process::Command::new("rsync")
        .arg("-a")
        .arg("--delete")
        .arg(format!("{}", profile_sync.source.display()))
        .arg(format!("{}", profile_sync.target.display()))
        .output()
        .map_err(|e| {
            spinner.finish();
            spinner.set_message("failed");
            Err(e)
        })?;

    spinner.finish();
    spinner.set_message("done");

    Ok(())
}
