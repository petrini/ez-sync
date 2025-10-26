use std::path::PathBuf;

pub struct Profile {
    source: PathBuf,
    target: PathBuf,
}

pub enum SyncMode {
    Push,
    Pull,
}

