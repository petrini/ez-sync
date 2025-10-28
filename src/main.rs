mod input;
mod processing;

use processing::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match input::validate_args()? {
        Command::List (profiles) =>
            for profile in profiles {
                println!("{}", profile);
            }
        _ => println!("Input validated"),
    }

    Ok(())
}
