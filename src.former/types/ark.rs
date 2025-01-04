//! The core Ark datastructure.

use crate::types::attrs::Attrs;
use crate::types::ipr::IPR;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Representation of an Archive.
///
/// Because this is generic, it can represent things like a directory on disk,
/// allowing us to convert an Ark of on-disk files into an Ark of imported files
/// in a simple, high-performance way. It obviates the need for things like a
/// stream API, and allows for a lot of tests to be done in-memory without disk.
///
/// The underlying format is an SOA approach, which you can inspect with:
///
///   - ark.paths()
///   - ark.attrs()
///   - ark.contents()
///
/// These three channels are implemented as immutable, reference-counted
/// vectors. This is great for memory hygiene! Almost every possible
/// transformation you'd ever want to do on an Ark will leave one or two
/// channels unchanged, and create a new vector for the stuff that _is_
/// changing.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Ark<C>(
    pub(crate) Rc<Vec<IPR>>,
    pub(crate) Rc<Vec<Attrs>>,
    pub(crate) Rc<Vec<C>>,
);

impl<C> Ark<C> {
    /// Internal paths list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F+D.
    ///  - There are no duplicate paths.
    ///  - All files come before all directories.
    ///  - Within each of those sections, paths are sorted.
    pub fn paths(&self) -> &Vec<IPR> {
        &self.0
    }

    /// Internal attrs list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F+D.
    ///  - `ark.attrs()[N]` corresponds to `ark.paths()[N]`.
    pub fn attrs(&self) -> &Vec<Attrs> {
        &self.1
    }

    /// Internal contents list.
    ///
    /// In an archive of length F+D, the following is guaranteed:
    ///
    ///  - This vector is length F, not F+D.
    ///  - `ark.contents()[N]` corresponds to `ark.paths()[N]`.
    pub fn contents(&self) -> &Vec<C> {
        &self.2
    }

    /// Number of entries in this Ark.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Iterate the files in an Archive
    pub fn files<'a>(&'a self) -> FileIterator<'a, C> {
        FileIterator {
            inner: &self,
            pos: 0,
        }
    }

    /// Iterate the dirs in an Archive
    pub fn dirs<'a>(&'a self) -> DirIterator<'a, C> {
        DirIterator {
            inner: &self,
            pos: self.0.len(),
        }
    }

    /// Slap together a new Ark from the constituent pieces.
    ///
    /// Panics if length invariants aren't fulfilled.
    pub fn compose(paths: Rc<Vec<IPR>>, attrs: Rc<Vec<Attrs>>, contents: Rc<Vec<C>>) -> Self {
        assert!(paths.len() == attrs.len());
        assert!(paths.len() >= contents.len());
        Self(paths, attrs, contents)
    }

    /// Break an Ark into its constituent components, moving them.
    ///
    /// This is designed to pair with `compose` to allow you to reuse backing
    /// memory while doing transformations. Usually you'll only care about
    /// transforming one, maybe two of the three channels.
    pub fn decompose(self) -> (Rc<Vec<IPR>>, Rc<Vec<Attrs>>, Rc<Vec<C>>) {
        (self.0, self.1, self.2)
    }

    /// Create an empty Ark.
    ///
    /// Not as widely useful as you'd think, since Ark is efficient for bulk
    /// operations, and not a great type for incremental mutability. Usually
    /// you'll want to work with a list of `(path, attrs, Contents<C>)` tuples for
    /// poking around in little bits and pieces. These convert back and forth
    /// with Arks very easily.
    pub fn empty() -> Self {
        Self::compose(Rc::new(vec![]), Rc::new(vec![]), Rc::new(vec![]))
    }
}

pub struct FileIterator<'a, C> {
    inner: &'a Ark<C>,
    pos: usize,
}
impl<'a, C> Iterator for FileIterator<'a, C> {
    type Item = (&'a IPR, &'a Attrs, &'a C);
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.inner.2.len() {
            None
        } else {
            let pos = self.pos;
            self.pos = pos + 1;
            Some((&self.inner.0[pos], &self.inner.1[pos], &self.inner.2[pos]))
        }
    }
}

pub struct DirIterator<'a, C> {
    inner: &'a Ark<C>,
    pos: usize,
}
impl<'a, C> Iterator for DirIterator<'a, C> {
    type Item = (&'a IPR, &'a Attrs);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == 0 {
            None
        } else {
            self.pos = self.pos - 1;
            Some((&self.inner.0[self.pos], &self.inner.1[self.pos]))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::ipr::ToIPR;

    #[test]
    fn test_files() {
        crate::setup_fixture();
        let ark = Ark::scan("fixture").expect("Scanned fixture");
        let mut files = ark.files();

        // Sorted order
        assert_eq!(
            files.next(),
            Some((
                &"dir1/dir2/nested.txt".to_ipr(),
                &Attrs::new().append("UNIX_MODE", "33204"),
                &std::path::PathBuf::from("fixture/dir1/dir2/nested.txt"),
            ))
        );
        assert_eq!(
            files.next(),
            Some((
                &"file_at_root.txt".to_ipr(),
                &Attrs::new().append("UNIX_MODE", "33204"),
                &std::path::PathBuf::from("fixture/file_at_root.txt"),
            ))
        );
    }

    #[test]
    fn test_dirs() {
        crate::setup_fixture();
        let ark = Ark::scan("fixture").expect("Scanned fixture");
        let mut dirs = ark.dirs();

        // Reverse sorted order.
        //
        // Consumers will often want to read these from most nested to least,
        // because applying permissions in any other order can lock yourself
        // out and make you unable to finish the job.
        assert_eq!(
            dirs.next(),
            Some((
                &"dir1/dir2".to_ipr(),
                &Attrs::new().append("UNIX_MODE", "16893"),
            ))
        );
        assert_eq!(
            dirs.next(),
            Some((&"dir1".to_ipr(), &Attrs::new().append("UNIX_MODE", "16893"),))
        );
    }
}
