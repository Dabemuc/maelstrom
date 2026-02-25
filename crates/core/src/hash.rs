use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

pub fn hash_file(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
