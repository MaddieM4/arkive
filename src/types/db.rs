//! On-disk database format.
//!
//! Very little behavior exists here, though this is a nice place to learn the
//! expected schema for what should exist in an Ark DB. It's tiny and simple, so
//! let's take a tour, eh?
//!
//! ```text
//! some/db/root <- (is usually a .dirtabase_db somewhere)
//! ├── cas
//! │   ├── 819d829... <- files named after their contents
//! │   ├── bbdef91...     (it's a content-addressed store)
//! │   ├── de572b2...
//! │   └── ff0213c...
//! ├── labels
//! │   ├── @mutable
//! │   ├── @human
//! │   ├── @readable_names <- these just contain hashes to CAS items
//! │   └── @are_good
//! ├── cache
//! │   ├── 282eebd... <- used by Dirtabase to remember the outputs
//! │   ├── 304baad...    of deterministic steps (and skip them next time)
//! │   └── 0380441...
//! └── tmp <- used briefly for certain operations, like imports
//! ```
//!
//! The reason so little of that behavior lives in this file, is... it lives
//! somewhere else! The DB object just needs to make a nice little empty home
//! for stuff to live. Putting stuff _into_ that home is the kind of job done by
//! Ark helper traits and the Dirtabase build system. Everybody's on the same
//! page, as far as what data should live where.

use std::io::Result;
use std::path::{Path, PathBuf};

/// Where persistent data lives.
pub enum DB {
    Persistent(PathBuf),
    Temp(tempfile::TempDir),
}

fn init_sections(p: &Path) -> Result<()> {
    for section in ["cas", "labels", "cache", "tmp"] {
        std::fs::create_dir_all(p.join(section))?;
    }
    Ok(())
}

impl DB {
    /// Initialize in a specific place.
    pub fn new(p: impl AsRef<Path>) -> Result<Self> {
        let p: PathBuf = p.as_ref().into();
        init_sections(p.as_ref())?;
        Ok(Self::Persistent(p))
    }

    /// Initialize in a temp directory. Deleted when this object is dropped.
    pub fn new_temp() -> Result<Self> {
        let t = tempfile::tempdir()?;
        init_sections(t.as_ref())?;
        Ok(Self::Temp(t))
    }

    /// Just a simple little path join.
    pub fn join(&self, p: impl AsRef<Path>) -> PathBuf {
        self.as_ref().join(p)
    }
}

impl AsRef<Path> for DB {
    fn as_ref(&self) -> &Path {
        match self {
            Self::Persistent(path) => path,
            Self::Temp(td) => td.as_ref(),
        }
    }
}
