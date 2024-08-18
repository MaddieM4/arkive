//! Read files from disk.
//!
//! ```
//! use arkive::*;
//!
//! let ark = Ark::scan("fixture")?.read()?;
//! # Ok::<(), std::io::Error>(())
//! ```

use crate::types::*;
use std::io::Result;
use std::path::PathBuf;
use std::rc::Rc;

impl Ark<PathBuf> {
    /// Fetch file contents from disk into memory.
    ///
    /// Be warned that this may be a very bad idea if the directory is larger
    /// than you have RAM for.
    pub fn read(self) -> Result<Ark<Vec<u8>>> {
        let (paths, attrs, contents) = self.decompose();
        let contents: Result<Vec<Vec<u8>>> = contents.iter().map(|pb| std::fs::read(&pb)).collect();
        Ok(Ark::compose(paths, attrs, Rc::new(contents?)))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn read() -> Result<()> {
        let ark = Ark::scan("fixture")?.read()?;
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
        assert_eq!(
            ark.contents(),
            &vec![
                "A file nested under multiple directories\n".into(),
                "Here are some file contents, teehee!\n".into(),
            ] as &Vec<Vec<u8>>
        );
        Ok(())
    }
}
