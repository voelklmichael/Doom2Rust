//! Read-only filesystem serving a single IWAD that is linked into the firmware image.

use alloc::{format, string::String};
use rust_doomgeneric::{DoomFileSystem, FileId};

/// Serves `data` under `name` (compared case-insensitively, ignoring any directory part).
/// Everything else does not exist and cannot be written, so the engine runs without a
/// config file, savegames or extra WADs.
pub struct EmbeddedWad {
    name: &'static str,
    data: &'static [u8],
}

impl EmbeddedWad {
    pub const fn new(name: &'static str, data: &'static [u8]) -> Self {
        Self { name, data }
    }

    fn matches(&self, path: &str) -> bool {
        let file_name = path.rsplit('/').next().unwrap_or(path);
        file_name.eq_ignore_ascii_case(self.name)
    }
}

impl DoomFileSystem for EmbeddedWad {
    fn open(&mut self, path: &str) -> Option<FileId> {
        self.matches(path).then_some(FileId(0))
    }

    fn close(&mut self, _file: FileId) {}

    fn len(&self, _file: FileId) -> u64 {
        self.data.len() as u64
    }

    fn read_at(&self, _file: FileId, offset: u64, buf: &mut [u8]) -> usize {
        let start = usize::try_from(offset).map_or(self.data.len(), |o| o.min(self.data.len()));
        let n = buf.len().min(self.data.len() - start);
        buf[..n].copy_from_slice(&self.data[start..start + n]);
        n
    }

    fn write_file(&mut self, _path: &str, _data: &[u8]) -> bool {
        false
    }

    fn remove_file(&mut self, _path: &str) -> bool {
        false
    }

    fn rename(&mut self, _from: &str, _to: &str) -> bool {
        false
    }

    fn exists(&self, path: &str) -> bool {
        self.matches(path)
    }

    fn create_dir(&mut self, _path: &str) {}

    fn temp_path(&self, name: &str) -> String {
        format!("/tmp/{name}")
    }
}
