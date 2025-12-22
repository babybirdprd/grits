use crate::fs::FileSystem;
use crate::git::GitOps;
use anyhow::{bail, Result};
use std::io::{BufRead, Cursor, Write};
use std::path::Path;
use wasm_bindgen::prelude::*;

// JS bindings for grits-core
#[wasm_bindgen(module = "/js/grits_fs.js")]
extern "C" {
    fn fs_read_to_string(path: &str) -> String;
    fn fs_write(path: &str, content: &[u8]);
    fn fs_create_dir_all(path: &str);
    fn fs_rename(from: &str, to: &str);
    fn fs_exists(path: &str) -> bool;
}

#[wasm_bindgen(module = "/js/grits_git.js")]
extern "C" {
    fn git_init() -> String;
    fn git_add(path: &str) -> String;
    fn git_commit(message: &str) -> String;
    fn git_pull_rebase() -> String;
    fn git_push() -> String;
    fn git_status() -> String;
    fn git_show(revision: &str) -> String;
    fn git_rebase_continue() -> String;
    fn git_has_remote() -> bool;
}

/// FileSystem implementation that delegates to JavaScript.
#[wasm_bindgen]
pub struct WasmFileSystem;

#[wasm_bindgen]
impl WasmFileSystem {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmFileSystem
    }
}

impl FileSystem for WasmFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        // In a real implementation, we'd handle errors from JS, possibly returning Result<String, JsValue>
        // and mapping it. For now assuming the JS binding throws or returns a string.
        // But wasm_bindgen extern functions usually match the signature.
        // If JS can fail, we should use catch.
        Ok(fs_read_to_string(path_str))
    }

    fn write(&self, path: &Path, contents: &[u8]) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs_write(path_str, contents);
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs_create_dir_all(path_str);
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        let from_str = from
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let to_str = to.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        fs_rename(from_str, to_str);
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        if let Some(path_str) = path.to_str() {
            fs_exists(path_str)
        } else {
            false
        }
    }

    fn open_read(&self, path: &Path) -> Result<Box<dyn BufRead>> {
        // For now, read entire file into memory and return a Cursor
        let content = self.read_to_string(path)?;
        Ok(Box::new(Cursor::new(content.into_bytes())))
    }

    fn open_write(&self, path: &Path) -> Result<Box<dyn Write>> {
        // This is tricky because we need a writer that writes back to JS on flush/drop.
        // For simple WASM usage, we might buffer in memory.
        Ok(Box::new(WasmFileWriter {
            path: path.to_path_buf(),
            buffer: Vec::new(),
            fs: WasmFileSystem,
        }))
    }
}

struct WasmFileWriter {
    path: std::path::PathBuf,
    buffer: Vec<u8>,
    fs: WasmFileSystem,
}

impl Write for WasmFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.fs
            .write(&self.path, &self.buffer)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(())
    }
}

#[wasm_bindgen]
pub struct WasmGit;

#[wasm_bindgen]
impl WasmGit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmGit
    }
}

impl GitOps for WasmGit {
    fn init(&self) -> Result<()> {
        let res = git_init();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn add(&self, path: &Path) -> Result<()> {
        let path_str = path.to_str().unwrap_or("");
        let res = git_add(path_str);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn commit(&self, message: &str) -> Result<()> {
        let res = git_commit(message);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn pull_rebase(&self) -> Result<()> {
        let res = git_pull_rebase();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn push(&self) -> Result<()> {
        let res = git_push();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn status(&self) -> Result<String> {
        let res = git_status();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(res)
    }

    fn show(&self, revision: &str) -> Result<String> {
        let res = git_show(revision);
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(res)
    }

    fn rebase_continue(&self) -> Result<()> {
        let res = git_rebase_continue();
        if res.starts_with("error:") {
            bail!("{}", res);
        }
        Ok(())
    }

    fn has_remote(&self) -> Result<bool> {
        Ok(git_has_remote())
    }
}
