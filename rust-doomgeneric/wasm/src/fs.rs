//! The game's files in the browser: the IWAD the player dropped on the page, and whatever the game
//! writes (savegames, the config). The written files live in memory, and every change is also
//! handed to a [`Persist`], which keeps them for the next visit.

use rust_doomgeneric::{DoomFileSystem, FileId};
use std::collections::BTreeMap;
use std::rc::Rc;

/// Where the files the game writes are kept between visits to the page.
pub trait Persist {
    fn store(&self, path: &str, data: &[u8]);
    fn remove(&self, path: &str);
}

enum Contents {
    Wad,
    File(Rc<[u8]>),
}

pub struct BrowserFs<P> {
    wad_name: &'static str,
    wad: Vec<u8>,
    files: BTreeMap<String, Rc<[u8]>>,
    persist: P,
    /// Open files, indexed by `FileId`. Closed slots are `None` and reused.
    open: Vec<Option<Contents>>,
}

impl<P: Persist> BrowserFs<P> {
    /// Serves `wad` under `wad_name` (compared case-insensitively, ignoring any directory part),
    /// and `saved`, the files kept from an earlier visit, under their own names.
    pub fn new(
        wad_name: &'static str,
        wad: Vec<u8>,
        saved: impl IntoIterator<Item = (String, Vec<u8>)>,
        persist: P,
    ) -> Self {
        Self {
            wad_name,
            wad,
            files: saved
                .into_iter()
                .map(|(path, data)| (path, data.into()))
                .collect(),
            persist,
            open: Vec::new(),
        }
    }

    fn is_wad(&self, path: &str) -> bool {
        let file_name = path.rsplit('/').next().unwrap_or(path);
        file_name.eq_ignore_ascii_case(self.wad_name)
    }

    fn bytes(&self, file: FileId) -> &[u8] {
        match self.open.get(file.0 as usize) {
            Some(Some(Contents::Wad)) => &self.wad,
            Some(Some(Contents::File(data))) => data,
            _ => &[],
        }
    }
}

impl<P: Persist> DoomFileSystem for BrowserFs<P> {
    fn open(&mut self, path: &str) -> Option<FileId> {
        let contents = if self.is_wad(path) {
            Contents::Wad
        } else {
            Contents::File(Rc::clone(self.files.get(path)?))
        };
        let slot = self.open.iter().position(Option::is_none);
        let index = slot.unwrap_or(self.open.len());
        if slot.is_none() {
            self.open.push(None);
        }
        self.open[index] = Some(contents);
        Some(FileId(index as u32))
    }

    fn close(&mut self, file: FileId) {
        if let Some(slot) = self.open.get_mut(file.0 as usize) {
            *slot = None;
        }
    }

    fn len(&self, file: FileId) -> u64 {
        self.bytes(file).len() as u64
    }

    fn read_at(&self, file: FileId, offset: u64, buf: &mut [u8]) -> usize {
        let data = self.bytes(file);
        let start = usize::try_from(offset).map_or(data.len(), |o| o.min(data.len()));
        let n = buf.len().min(data.len() - start);
        buf[..n].copy_from_slice(&data[start..start + n]);
        n
    }

    fn write_file(&mut self, path: &str, data: &[u8]) -> bool {
        if self.is_wad(path) {
            return false;
        }
        self.files.insert(path.to_string(), data.into());
        self.persist.store(path, data);
        true
    }

    fn remove_file(&mut self, path: &str) -> bool {
        let removed = self.files.remove(path).is_some();
        if removed {
            self.persist.remove(path);
        }
        removed
    }

    fn rename(&mut self, from: &str, to: &str) -> bool {
        match self.files.remove(from) {
            Some(data) => {
                self.persist.remove(from);
                self.persist.store(to, &data);
                self.files.insert(to.to_string(), data);
                true
            }
            None => false,
        }
    }

    fn exists(&self, path: &str) -> bool {
        self.is_wad(path) || self.files.contains_key(path)
    }

    /// Directories are not tracked: any path can be written to.
    fn create_dir(&mut self, _path: &str) {}

    fn temp_path(&self, name: &str) -> String {
        format!("/tmp/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    static WAD: &[u8] = b"IWAD0123456789";

    /// The files, and a log of what the persister was asked to do.
    type Kept = (BTreeMap<String, Vec<u8>>, Vec<String>);

    #[derive(Clone, Default)]
    struct Recorder(Rc<RefCell<Kept>>);

    impl Persist for Recorder {
        fn store(&self, path: &str, data: &[u8]) {
            let mut inner = self.0.borrow_mut();
            inner.0.insert(path.to_string(), data.to_vec());
            inner.1.push(format!("store {path}"));
        }
        fn remove(&self, path: &str) {
            let mut inner = self.0.borrow_mut();
            inner.0.remove(path);
            inner.1.push(format!("remove {path}"));
        }
    }

    fn new_fs(saved: &[(&str, &[u8])]) -> (BrowserFs<Recorder>, Recorder) {
        let recorder = Recorder::default();
        let saved = saved
            .iter()
            .map(|(path, data)| (path.to_string(), data.to_vec()));
        let fs = BrowserFs::new("doom1.wad", WAD.to_vec(), saved, recorder.clone());
        (fs, recorder)
    }

    fn read_all(fs: &BrowserFs<Recorder>, file: FileId) -> Vec<u8> {
        let mut data = vec![0; fs.len(file) as usize];
        assert_eq!(fs.read_at(file, 0, &mut data), data.len());
        data
    }

    #[test]
    fn wad_is_found_by_file_name_ignoring_case_and_directory() {
        let (mut fs, _) = new_fs(&[]);
        assert!(fs.exists("DOOM1.WAD"));
        assert!(fs.exists("/usr/share/games/doom/Doom1.wad"));
        assert!(!fs.exists("doom2.wad"));
        let file = fs.open("./doom1.wad").unwrap();
        assert_eq!(read_all(&fs, file), WAD);
        let mut buf = [0; 4];
        assert_eq!(fs.read_at(file, 10, &mut buf), 4);
        assert_eq!(&buf, b"6789");
        assert_eq!(fs.read_at(file, 12, &mut buf), 2);
        assert_eq!(fs.read_at(file, 99, &mut buf), 0);
    }

    #[test]
    fn written_files_can_be_read_renamed_and_removed_but_the_wad_is_read_only() {
        let (mut fs, _) = new_fs(&[]);
        assert!(fs.open("save0.dsg").is_none());
        assert!(fs.write_file("save0.dsg", b"saved"));
        assert!(!fs.write_file("doom1.wad", b"overwritten"));
        let file = fs.open("save0.dsg").unwrap();
        assert_eq!(read_all(&fs, file), b"saved");
        // An open file keeps its contents when the file is replaced.
        assert!(fs.write_file("save0.dsg", b"newer"));
        assert_eq!(read_all(&fs, file), b"saved");
        fs.close(file);
        assert!(fs.rename("save0.dsg", "save1.dsg"));
        assert!(!fs.exists("save0.dsg"));
        let file = fs.open("save1.dsg").unwrap();
        // The slot of the closed file is reused.
        assert_eq!(file, FileId(0));
        assert_eq!(read_all(&fs, file), b"newer");
        assert!(fs.remove_file("save1.dsg"));
        assert!(!fs.remove_file("save1.dsg"));
    }

    #[test]
    fn files_from_an_earlier_visit_are_there_and_every_change_is_persisted() {
        let (mut fs, recorder) = new_fs(&[("doomsav0.dsg", b"old")]);
        let file = fs.open("doomsav0.dsg").unwrap();
        assert_eq!(read_all(&fs, file), b"old");
        // What was restored is not written back.
        assert!(recorder.0.borrow().1.is_empty());

        assert!(fs.write_file("temp.dsg", b"new"));
        assert!(fs.rename("temp.dsg", "doomsav1.dsg"));
        assert!(fs.remove_file("doomsav0.dsg"));
        // Nothing is persisted for a change that did not happen.
        assert!(!fs.remove_file("doomsav0.dsg"));
        assert!(!fs.rename("missing", "other"));
        assert!(!fs.write_file("DOOM1.WAD", b"x"));

        let (files, log) = &*recorder.0.borrow();
        assert_eq!(
            log,
            &[
                "store temp.dsg",
                "remove temp.dsg",
                "store doomsav1.dsg",
                "remove doomsav0.dsg"
            ]
        );
        assert_eq!(files.keys().collect::<Vec<_>>(), ["doomsav1.dsg"]);
    }
}
