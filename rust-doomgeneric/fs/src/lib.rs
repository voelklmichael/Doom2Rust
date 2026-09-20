//! `std::fs`-backed implementation of the engine's `DoomFileSystem` trait.
//! All of the game's real filesystem access lives here so that the engine
//! crate itself can stay free of `std::fs`/`std::io`.

use ::rust_doomgeneric::{DoomFileSystem, FileId};
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, FileExt};

const EISDIR: i32 = 21;

#[derive(Default)]
pub struct StdFileSystem {
    /// Open files, indexed by `FileId`. Closed slots are `None` and reused.
    open: Vec<Option<File>>,
}

impl StdFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    fn file(&self, id: FileId) -> Option<&File> {
        self.open.get(id.0 as usize)?.as_ref()
    }
}

impl DoomFileSystem for StdFileSystem {
    fn open(&mut self, path: &str) -> Option<FileId> {
        let file = File::open(path).ok()?;
        let index = match self.open.iter().position(Option::is_none) {
            Some(free) => {
                self.open[free] = Some(file);
                free
            }
            None => {
                self.open.push(Some(file));
                self.open.len() - 1
            }
        };
        Some(FileId(index as u32))
    }

    fn close(&mut self, file: FileId) {
        if let Some(slot) = self.open.get_mut(file.0 as usize) {
            *slot = None;
        }
    }

    fn len(&self, file: FileId) -> u64 {
        self.file(file)
            .and_then(|f| f.metadata().ok())
            .map_or(0, |m| m.len())
    }

    fn read_at(&self, file: FileId, offset: u64, buf: &mut [u8]) -> usize {
        self.file(file)
            .and_then(|f| f.read_at(buf, offset).ok())
            .unwrap_or(0)
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> bool {
        match File::create(path) {
            Ok(mut handle) => handle.write_all(data).is_ok(),
            Err(_) => false,
        }
    }

    fn remove_file(&mut self, path: &str) -> bool {
        std::fs::remove_file(path).is_ok()
    }

    fn rename(&mut self, from: &str, to: &str) -> bool {
        std::fs::rename(from, to).is_ok()
    }

    fn exists(&self, path: &str) -> bool {
        match File::open(path) {
            Ok(_) => true,
            Err(e) => e.raw_os_error() == Some(EISDIR),
        }
    }

    fn create_dir(&mut self, path: &str) {
        let _ = std::fs::DirBuilder::new().mode(0o755).create(path);
    }

    fn temp_path(&self, name: &str) -> String {
        format!("/tmp/{name}")
    }
}
