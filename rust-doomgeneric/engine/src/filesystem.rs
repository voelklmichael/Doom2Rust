use alloc::string::String;
use alloc::vec::Vec;
/// Handle to a file opened through a [`DoomFileSystem`]. Only meaningful to
/// the filesystem that returned it, and only until it is passed to `close`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FileId(pub u32);

/// Everything the engine needs from the host's filesystem. The engine itself
/// never touches `std::fs`; the host (see the `doomgeneric_fs` crate) hands in
/// an implementation, exactly like [`DoomPlatform`](crate::DoomPlatform).
pub trait DoomFileSystem {
    /// Opens `path` for random-access reading.
    fn open(&mut self, path: &str) -> Option<FileId>;
    fn close(&mut self, file: FileId);
    fn len(&self, file: FileId) -> u64;
    /// Reads up to `buf.len()` bytes starting at `offset`; returns the number
    /// of bytes read (0 on error or at end of file).
    fn read_at(&self, file: FileId, offset: u64, buf: &mut [u8]) -> usize;
    /// Creates or truncates `path` and writes all of `data`.
    fn write_file(&mut self, path: &str, data: &[u8]) -> bool;
    fn remove_file(&mut self, path: &str) -> bool;
    fn rename(&mut self, from: &str, to: &str) -> bool;
    /// Whether `path` names something that exists (a directory counts).
    fn exists(&self, path: &str) -> bool;
    fn create_dir(&mut self, path: &str);
    /// Path of a scratch file called `name` in the host's temp directory.
    fn temp_path(&self, name: &str) -> String;
}

/// Reads all of `path` into memory.
pub fn read_file(fs: &mut dyn DoomFileSystem, path: &str) -> Option<Vec<u8>> {
    let file = fs.open(path)?;
    let mut data = vec![0u8; fs.len(file) as usize];
    let mut filled = 0;
    while filled < data.len() {
        let n = fs.read_at(file, filled as u64, &mut data[filled..]);
        if n == 0 {
            break;
        }
        filled += n;
    }
    data.truncate(filled);
    fs.close(file);
    Some(data)
}

#[cfg(test)]
pub use mem::MemFileSystem;

#[cfg(test)]
mod mem {
    use super::{DoomFileSystem, FileId};
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;
    use std::collections::BTreeMap;

    /// In-memory filesystem for tests: no disk access, and directories are
    /// only tracked as names.
    #[derive(Default)]
    pub struct MemFileSystem {
        pub files: BTreeMap<String, Vec<u8>>,
        pub dirs: Vec<String>,
        open: Vec<Option<Vec<u8>>>,
    }

    impl DoomFileSystem for MemFileSystem {
        fn open(&mut self, path: &str) -> Option<FileId> {
            let data = self.files.get(path)?.clone();
            let slot = self.open.iter().position(Option::is_none);
            Some(FileId(match slot {
                Some(i) => {
                    self.open[i] = Some(data);
                    i as u32
                }
                None => {
                    self.open.push(Some(data));
                    (self.open.len() - 1) as u32
                }
            }))
        }
        fn close(&mut self, file: FileId) {
            self.open[file.0 as usize] = None;
        }
        fn len(&self, file: FileId) -> u64 {
            self.open[file.0 as usize].as_ref().unwrap().len() as u64
        }
        fn read_at(&self, file: FileId, offset: u64, buf: &mut [u8]) -> usize {
            let data = self.open[file.0 as usize].as_ref().unwrap();
            let start = (offset as usize).min(data.len());
            let n = buf.len().min(data.len() - start);
            buf[..n].copy_from_slice(&data[start..start + n]);
            n
        }
        fn write_file(&mut self, path: &str, data: &[u8]) -> bool {
            self.files.insert(path.to_string(), data.to_vec());
            true
        }
        fn remove_file(&mut self, path: &str) -> bool {
            self.files.remove(path).is_some()
        }
        fn rename(&mut self, from: &str, to: &str) -> bool {
            match self.files.remove(from) {
                Some(data) => {
                    self.files.insert(to.to_string(), data);
                    true
                }
                None => false,
            }
        }
        fn exists(&self, path: &str) -> bool {
            self.files.contains_key(path) || self.dirs.iter().any(|d| d == path)
        }
        fn create_dir(&mut self, path: &str) {
            self.dirs.push(path.to_string());
        }
        fn temp_path(&self, name: &str) -> String {
            format!("/tmp/{name}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_returns_whole_contents_and_closes() {
        let mut fs = MemFileSystem::default();
        fs.files.insert("a.dsg".into(), (0..=255u8).collect());
        assert_eq!(read_file(&mut fs, "a.dsg"), Some((0..=255u8).collect()));
        assert_eq!(read_file(&mut fs, "missing.dsg"), None);
        // The slot from the first read was released and is reused.
        assert_eq!(fs.open("a.dsg"), Some(FileId(0)));
    }
}
