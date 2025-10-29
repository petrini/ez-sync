mod input;
mod config;
mod profile;

use profile::{Profile, Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = input::parse_args();
    let config_path = config::get_path(&args.config)?;
    let config = config::Config::load(&config_path)?;

    match input::validate_command(config, args.command)? {
        Command::Add(mut config, mut profile) => {
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
            };

            config.save(config_path)?;
            profile.name = full_name;
            println!("Profile added");
            println!("{}", profile);
            return Ok(())
        }
        Command::Remove(mut config, profile) => {
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
            return Ok(())
        }
        Command::Sync(profile_syncs) =>
            for profile_sync in profile_syncs {
                println!("SYNC src: {} tgt: {}", profile_sync.source.display(), profile_sync.target.display());
            }
        Command::List (profiles) => list_profiles(&profiles),
    }

    Ok(())
}

fn list_profiles(profiles: &Vec<Profile>) {
    for profile in profiles {
        println!("{}", profile);
    }
}
