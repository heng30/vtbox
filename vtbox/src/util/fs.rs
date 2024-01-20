use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

const G: u64 = 1024 * 1024 * 1024;
const M: u64 = 1024 * 1024;
const K: u64 = 1024;

#[allow(dead_code)]
pub fn working_dir() -> Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();

    match dir.to_str() {
        Some(path) => Ok(PathBuf::from(path)),
        _ => Err(anyhow::anyhow!("convert {:?} failed", dir)),
    }
}

#[allow(dead_code)]
pub fn remove_dir_files(path: &str) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn filename(path: &str) -> Option<String> {
    match Path::new(path).file_name() {
        None => None,
        Some(v) => v.to_str().map(|v| v.to_string()),
        // Some(v) => match v.to_str() {
        //     None => None,
        //     Some(v) => Some(v.to_string()),
        // },
    }
}

#[allow(dead_code)]
pub fn path_size(path: &str) -> Result<u64> {
    let mut total_size: u64 = 0;
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        for entry in fs::read_dir(path)?.flatten() {
            let size = entry.metadata()?.len();
            total_size += size;
        }
    } else {
        total_size += metadata.len();
    }

    Ok(total_size)
}

#[allow(dead_code)]
pub fn file_size(path: &str) -> Result<u64> {
    let metadata = fs::metadata(path)?;

    Ok(if metadata.is_file() {
        metadata.len()
    } else {
        0
    })
}

#[allow(dead_code)]
pub fn file_exist(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(md) => md.is_file(),
        _ => false,
    }
}

#[allow(dead_code)]
pub fn pretty_size(size: u64) -> String {
    if size >= G {
        format!("{:.2}G", size as f64 / G as f64)
    } else if size >= M {
        format!("{:}M", size / M)
    } else {
        format!("{:}K", size / K)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_dir() -> Result<()> {
        let wd = working_dir()?;
        // println!("{:?}", wd);
        assert!(wd.is_dir());

        Ok(())
    }

    #[test]
    fn test_filename() -> Result<()> {
        let path = "/home/user/Downloads/filename.ext";
        let name = filename(path).unwrap();
        assert_eq!("filename.ext", name);

        let path = "filename.ext";
        let name = filename(path).unwrap();
        assert_eq!("filename.ext", name);

        assert!(filename("").is_none());

        Ok(())
    }

    #[test]
    fn test_path_size() -> Result<()> {
        let size = path_size(".")?;
        assert!(size > 0);

        Ok(())
    }

    #[test]
    fn test_file_size() -> Result<()> {
        let size = file_size("Cargo.toml")?;
        assert!(size > 0);

        Ok(())
    }

    #[test]
    fn test_pretty_size() -> Result<()> {
        let size = pretty_size(1024 * 1024 * 1024);
        assert_eq!(size, "1.00G");

        let size = pretty_size(1024 * 1024);
        assert_eq!(size, "1M");

        let size = pretty_size(1024);
        assert_eq!(size, "1K");

        Ok(())
    }
}
