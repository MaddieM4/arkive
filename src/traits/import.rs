//! Import files into a DB.
//!
//! ```
//! use arkive::*;
//! let db = DB::new_temp()?;
//!
//! // Imports all files, then the serialied archive. Gives you the CAS address.
//! let digest = Ark::scan("src")?.import(&db)?;
//! # Ok::<(), std::io::Error>(())
//! ```
use crate::traits::save::Save;
use crate::types::*;
use std::io::Result;
use std::iter::zip;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tempfile::{tempdir_in, TempDir};

pub trait Temporizable {
    fn temporize(&self, dest: &Path) -> Result<()>;
}

impl Temporizable for PathBuf {
    fn temporize(&self, dest: &Path) -> Result<()> {
        std::fs::copy(self, dest)?;
        Ok(())
    }
}
impl Temporizable for Vec<u8> {
    fn temporize(&self, dest: &Path) -> Result<()> {
        std::fs::write(dest, self)?;
        Ok(())
    }
}

fn temporize_files<T>(db: &DB, contents: &Vec<T>) -> Result<(TempDir, Vec<PathBuf>)>
where
    T: Temporizable,
{
    let dir = tempdir_in(db.join("tmp"))?;
    let temps: Result<Vec<PathBuf>> = contents
        .iter()
        .enumerate()
        .map(|(n, t)| {
            let dest = dir.as_ref().join(n.to_string());
            t.temporize(&dest)?;
            Ok(dest)
        })
        .collect();
    Ok((dir, temps?))
}

fn hash_file(pb: &PathBuf) -> Result<Digest> {
    let f = std::fs::File::open(pb)?;

    if f.metadata()?.len() == 0 {
        // Unfortunately it's an error to map an empty file.
        // It's deeply obnoxious to need a second metadata call here.
        // Maybe a solution will eventually present itself, or perhaps when
        // actually benched, the cost of this op is trivial. Hard to say!
        return Ok(Digest::from(""));
    }

    let mmap = unsafe { memmap::Mmap::map(&f)? };
    Ok(Digest::from(mmap.as_ref()))
}

fn hash_files(paths: &Vec<PathBuf>) -> Result<Vec<Digest>> {
    // TODO: Parallelize with Rayon, compare speed
    paths.iter().map(|pb| hash_file(pb)).collect()
}

impl<C> Ark<C>
where
    C: Temporizable,
{
    /// Import files into an on-disk database.
    pub fn import_files(self, db: &DB) -> Result<Ark<Digest>> {
        let (paths, attrs, contents) = self.decompose();
        let (_dir, temps) = temporize_files(db, &contents)?;
        let digests = hash_files(&temps)?;
        for (temp, digest) in zip(temps, &digests) {
            std::fs::rename(temp, db.join("cas").join(digest.to_hex()))?;
        }
        Ok(Ark::compose(paths, attrs, Rc::new(digests)))
    }

    /// Import files _and_ serialized self into DB.
    pub fn import(self, db: &DB) -> Result<Digest> {
        self.import_files(db)?.save(db)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn import_files() -> Result<()> {
        let db = DB::new_temp()?;
        let ark = Ark::scan("fixture")?.import_files(&db)?;

        assert_eq!(
            ark.paths(),
            &vec![
                "dir1/dir2/nested.txt",
                "file_at_root.txt",
                "dir1",
                "dir1/dir2",
            ]
        );
        assert_eq!(
            ark.attrs(),
            &vec![
                at! { UNIX_MODE => "33204" },
                at! { UNIX_MODE => "33204" },
                at! { UNIX_MODE => "16893" },
                at! { UNIX_MODE => "16893" },
            ]
        );

        let expected_text = "A file nested under multiple directories\n";
        let d = ark.contents()[0];
        let p = db.as_ref().join("cas").join(d.to_hex());

        assert_eq!(d, Digest::from(expected_text));
        assert_eq!(std::fs::read_to_string(p)?, expected_text.to_owned());

        // Get same results if we start from in-mem copy
        assert_eq!(Ark::scan("fixture")?.read()?.import_files(&db)?, ark);

        Ok(())
    }

    #[test]
    fn import() -> Result<()> {
        let db = DB::new_temp()?;
        let digest = Ark::scan("fixture")?.import(&db)?;
        assert_eq!(
            digest.to_hex(),
            "fb9dde674e4002c7646770fcdee7eb2669de2aa90b216f47331f7bd155d0f787"
        );
        Ok(())
    }

    #[test]
    fn empty_files() -> Result<()> {
        let db = DB::new_temp()?;
        let digest = Ark::scan("src")?.import(&db);
        assert!(digest.is_ok());
        Ok(())
    }
}
