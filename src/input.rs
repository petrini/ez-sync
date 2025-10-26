use std::path::PathBuf;
use std::error::Error;
use clap::Parser;

use crate::processing::{Profile, SyncMode};

const PROFILES_CONFIG: &str = "profiles.toml";

pub struct ValidInput {
    pub profile: Profile,
    pub sync_mode: SyncMode,
    pub force: bool,
    pub rsync_params: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub profile: String,
    #[arg(long, default_value_t = true)]
    pub push: bool,
    #[arg(long, default_value_t = false)]
    pub pull: bool,
    #[arg(long, default_value_t = false)]
    pub force: bool,
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub rsync_params: Option<String>,
    #[arg(short, long, default_value_t = false)]
    pub help: bool,
}

pub fn validate_args() -> Result<Option<ValidInput>, Box<dyn Error>> {
    let args = Args::parse();

    if args.help {
        return Ok(None);
    }

    let config = args.config.ok_or({
        match dirs::config_dir() {
            Some(config_dir) => {
                let config_file = config_dir;
                config_file.push(PROFILES_CONFIG);
                config_file
            }
            None => {
                return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("Config dir not found, specify with --config <config>"))));
            }
        }

    });
    Ok(Some(ValidInput {}))
}

fn print_usage() {
    let exe = std::env::args()
        .next()
        .unwrap_or_default();
    println!();
    println!("Usage: {} <profile>", exe);
    println!();
    println!("Options:");
    println!("\t--push (implicit) syncs profile files from source to target, mutually exclusive with pull");
    println!("\t--pull syncs profile files from target to source, mutually exclusive with push");
    println!("\t--force forces operation when destination has newer files");
    println!("\t--config <config> uses <config> file instead of default ~/.config/ez-sync/profiles.toml");
    println!("\t--sync-param <params-str> uses <param-str> for rsync instead of default -avh --delete");
    println!("\t-h / --help prints this help");
    println!();
}

