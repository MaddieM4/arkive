use crate::types::*;
use std::fs::{copy, create_dir_all};
use std::io::Result;
use std::path::{Path, PathBuf};

impl Ark<PathBuf> {
    /// Write files to a directory.
    ///
    /// TODO: Permissions
    pub fn write(&self, dest: impl AsRef<Path>) -> Result<()> {
        let p = dest.as_ref();
        for (ipr, _, contents) in self.files() {
            let dest_file = p.join(ipr.as_ref());
            match dest_file.parent() {
                Some(parent_dir) => create_dir_all(parent_dir)?,
                None => (),
            }
            copy(contents, dest_file)?;
        }

        for (ipr, _) in self.dirs() {
            let dest_dir = p.join(ipr.as_ref());
            if !dest_dir.exists() {
                create_dir_all(dest_dir)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write() -> Result<()> {
        let td = tempfile::tempdir()?;

        // I really need a test case with empty directories.
        // I'll bodge one up for now.
        let mut entries = Ark::scan("fixture")?.to_entries();
        entries.push(("dir1/dir2/emptydir".into(), Attrs::new(), Contents::Dir));
        let ark = Ark::from_entries(entries);

        // Well, does it work?
        ark.write(&td)?;
        assert!(td.path().join("dir1/dir2/nested.txt").exists());
        assert!(td.path().join("dir1/dir2/emptydir").exists());
        Ok(())
    }
}
