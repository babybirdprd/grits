use anyhow::Result;
use std::io::{BufRead, Write};
use std::path::Path;

pub trait FileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String>;
    fn write(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn open_read(&self, path: &Path) -> Result<Box<dyn BufRead>>;
    fn open_write(&self, path: &Path) -> Result<Box<dyn Write>>;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct StdFileSystem;

#[cfg(not(target_arch = "wasm32"))]
impl FileSystem for StdFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        std::fs::write(path, contents)?;
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        std::fs::rename(from, to)?;
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn open_read(&self, path: &Path) -> Result<Box<dyn BufRead>> {
        use std::io::BufReader;
        let file = std::fs::File::open(path)?;
        Ok(Box::new(BufReader::new(file)))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn Write>> {
        let file = std::fs::File::create(path)?;
        Ok(Box::new(file))
    }
}
