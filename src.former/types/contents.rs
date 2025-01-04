//! An enum used in the per-entry representation of archives.

/// An enum we use to differentiate dirs vs files.
///
/// File content is represented flexibly, and can be anything consistent,
/// from in-memory strings to digests that represent stored data. That's
/// the secret sauce for performance and clarity when it comes to tasks
/// like importing and exporting files from a store with massive parallelism.
#[derive(Debug, PartialEq)]
pub enum Contents<C> {
    Dir,
    File(C),
}

impl<C> Contents<C> {
    /// Does this represent a directory?
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Dir => true,
            Self::File(_) => false,
        }
    }

    /// Does this represent a file?
    pub fn is_file(&self) -> bool {
        match self {
            Self::Dir => false,
            Self::File(_) => true,
        }
    }
}
