// PathBuf

use std::fs::{DirEntry, Metadata, ReadDir};

pub struct Scanner<S>
where
    S: Fn(DirEntry) -> bool,
{
    stack: Vec<ReadDir>,
    criteria: S,
}

impl<S> Scanner<S>
where
    S: Fn(std::fs::DirEntry) -> bool,
{
    pub fn new(path: impl AsRef<std::path::Path>, criteria: S) -> std::io::Result<Self> {
        let rd = std::fs::read_dir(path)?;
        Ok(Self {
            stack: vec![rd],
            criteria: criteria,
        })
    }

    fn consider(
        &mut self,
        item: std::io::Result<DirEntry>,
    ) -> std::io::Result<(Metadata, DirEntry)> {
        let de = item?;
        let m = de.metadata()?;
        if m.is_dir() {
            let rd = std::fs::read_dir(de.path())?;
            self.stack.push(rd);
        }
        Ok((m, de))
    }
}

impl<S> Iterator for Scanner<S>
where
    S: Fn(DirEntry) -> bool,
{
    type Item = std::io::Result<(Metadata, DirEntry)>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut rd) = self.stack.pop() {
            if let Some(item) = rd.next() {
                self.stack.push(rd);
                return Some(self.consider(item));
            }
        }
        return None;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scan_fixture() -> std::io::Result<()> {
        let mut v = Scanner::new("./fixture", |_| true)?
            .map(|mde| Ok(mde?.1.path().to_string_lossy().to_string()))
            .collect::<std::io::Result<Vec<String>>>()?;
        v.sort();
        assert_eq!(
            v,
            vec![
                "./fixture/dir1",
                "./fixture/dir1/dir2",
                "./fixture/dir1/dir2/nested.txt",
                "./fixture/file_at_root.txt",
            ]
        );
        Ok(())
    }
}
