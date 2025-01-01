// PathBuf

use std::fs::{DirEntry, Metadata, ReadDir};

fn visit(
    root: impl AsRef<std::path::Path>,
    mut predicate: impl FnMut(&Metadata, &DirEntry) -> bool,
) -> std::io::Result<()> {
    let mut stack: Vec<ReadDir> = vec![std::fs::read_dir(root)?];
    while let Some(mut rd) = stack.pop() {
        if let Some(opt_de) = rd.next() {
            stack.push(rd);
            let de = opt_de?;
            let m = de.metadata()?;
            let should_descend = predicate(&m, &de);
            if m.is_dir() && should_descend {
                stack.push(std::fs::read_dir(de.path())?);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_visit() -> std::io::Result<()> {
        let mut v: Vec<String> = vec![];
        visit("./fixture", |_, de| {
            v.push(de.path().to_string_lossy().into());
            true
        })?;

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

    #[test]
    fn test_visit_criteria() -> std::io::Result<()> {
        let mut v: Vec<String> = vec![];
        visit("./fixture", |_, de| {
            if de.file_name() == "dir2" {
                false
            } else {
                v.push(de.path().to_string_lossy().into());
                true
            }
        })?;

        v.sort();
        assert_eq!(v, vec!["./fixture/dir1", "./fixture/file_at_root.txt",]);
        Ok(())
    }
}
