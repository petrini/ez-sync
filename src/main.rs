mod input;
mod config;
mod profile;

use profile::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match input::validate_args()? {
        Command::Add (_, profile) =>
            println!("ADD {}", profile),
        Command::Remove(_, profile) =>
            println!("REMOVE {}", profile),
        Command::Sync (profile_syncs) =>
            for profile_sync in profile_syncs {
                println!("SYNC src: {} tgt: {}", profile_sync.source.display(), profile_sync.target.display());
            }
        Command::List (profiles) =>
            for profile in profiles {
                println!("{}", profile);
            }
    }

    Ok(())
}
