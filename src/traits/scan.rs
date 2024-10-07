//! Read file metadata from disk. See `.read()`.

use crate::types::*;
use std::io::{Error, Result};
use std::path::{Path, PathBuf};

/// Read the filesystem to create a list of entries for a given directory.
///
/// The content type is PathBuf to allow easy reading later. They represent
/// where the corresponding files live on your actual FS. So you can mess with
/// the paths in memory to your heart's content - if you want - and they'll
/// still read correctly when you read them.
///
/// One particular use case for messing around between scan and read? Filtering
/// out stuff you don't want to include _before_ you import it. Smart. Even so,
/// you'd probably prefer to do that with a smarter scan process that doesn't
/// recurse into ignored directories and _then_ filter them out. I might
/// implement that later.
pub fn scan_to_entries(base: impl AsRef<Path>) -> Result<Vec<(IPR, Attrs, Contents<PathBuf>)>> {
    let mut output: Vec<(IPR, Attrs, Contents<PathBuf>)> = vec![];
    _scan(base.as_ref(), base.as_ref(), &mut output)?;
    Ok(output)
}

fn _scan(base: &Path, cur: &Path, output: &mut Vec<(IPR, Attrs, Contents<PathBuf>)>) -> Result<()> {
    if cur.is_dir() {
        for entry in std::fs::read_dir(cur)? {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;
            let ipr = _relativize_path(base, &path)?;

            if meta.is_dir() {
                _scan(base, &path, output)?;
                output.push((ipr, meta.into(), Contents::Dir));
            } else {
                output.push((ipr, meta.into(), Contents::File(path)));
            }
        }
    }
    Ok(())
}
fn _relativize_path(base: &Path, p: &Path) -> Result<IPR> {
    p.strip_prefix(&base)
        .unwrap()
        .try_into()
        .map_err(|e| Error::other(e))
}

impl Ark<PathBuf> {
    /// Fetch metadata for a directory into memory.
    ///
    /// This isn't a parallel process, but it's fast, and allows subsequent
    /// steps to load in a high-performance parallel way.
    pub fn scan(base: impl AsRef<Path>) -> Result<Self> {
        Ok(scan_to_entries(base)?.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn scan() -> Result<()> {
        crate::setup_fixture();
        let ark = Ark::scan("fixture")?;

        /*
        fixture
        ├── dir1
        │   └── dir2
        │       └── nested.txt
        └── file_at_root.txt
        */

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
        assert_eq!(ark.contents().len(), 2);
        Ok(())
    }
}
