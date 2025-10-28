mod input;
mod processing;

use processing::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match input::validate_args()? {
        Command::Sync (profile_syncs) =>
            for profile_sync in profile_syncs {
                println!("src: {} tgt: {}", profile_sync.source.display(), profile_sync.target.display());
            }
        Command::List (profiles) =>
            for profile in profiles {
                println!("{}", profile);
            }
        _ => println!("Input validated"),
    }

    Ok(())
}
