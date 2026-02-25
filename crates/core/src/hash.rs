use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

pub fn hash_file(path: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let filename = path
        .file_name()
        .ok_or("Path has no filename")?
        .to_string_lossy();

    let mut hasher = Sha256::new();

    // Include filename in hash
    hasher.update(filename.as_bytes());

    // Include file contents in hash
    hasher.update(&bytes);

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
