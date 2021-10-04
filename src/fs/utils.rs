use std::fs as stdfs;
use std::io::Result;
use std::path::Path;

pub fn get_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let meta = stdfs::metadata(&path)?;
    if !meta.is_dir() {
        return Ok(meta.len());
    }

    let mut size: u64 = 0;

    for entry in stdfs::read_dir(&path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            size += get_size(path)?;
        } else {
            size += path.metadata()?.len();
        }
    }

    return Ok(size);
}
