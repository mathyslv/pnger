use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Read a file from disk
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
