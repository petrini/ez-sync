mod input;
mod processing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    input::validate_args()?;
    println!("Input validated");
    Ok(())
}
