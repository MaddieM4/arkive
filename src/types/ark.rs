use crate::types::entry::{to_entries, Content, Entry, ToEntry};
use crate::types::ipr::IPR;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Ark<C, M = ()> {
    paths: Rc<Vec<IPR>>,
    metas: Rc<Vec<M>>,
    files: Rc<Vec<C>>,
    links: Rc<Vec<String>>,
}

impl<C, M> Ark<C, M> {
    // This conversion isn't as cheap as I'd like, but I don't
    // want to mess with it further until that's a problem in
    // the profiler. Odds of being a real bottleneck: low.
    pub fn from_entries<SRC>(src: SRC) -> Self
    where
        SRC: IntoIterator,
        SRC::Item: ToEntry<Content = C, Metadata = M>,
    {
        let uniq: HashMap<IPR, (M, Content<C>)> =
            to_entries(src).map(|(p, m, c)| (p, (m, c))).collect();

        let mut entries: Vec<Entry<C, M>> = uniq.into_iter().map(|(p, (m, c))| (p, m, c)).collect();

        // We get everything in path order, then don't break that
        // internal ordering when we sort into groups.
        entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        entries.sort_by_key(|(_, _, c)| match c {
            Content::File(_) => 0,
            Content::Symlink(_) => 1,
            Content::Directory => 2,
        });

        let mut paths: Vec<IPR> = vec![];
        let mut metas: Vec<M> = vec![];
        let mut files: Vec<C> = vec![];
        let mut links: Vec<String> = vec![];

        for (p, m, c) in entries {
            paths.push(p);
            metas.push(m);
            match c {
                Content::File(content) => files.push(content),
                Content::Symlink(s) => links.push(s),
                Content::Directory => (),
            };
        }

        Self {
            paths: Rc::new(paths),
            metas: Rc::new(metas),
            files: Rc::new(files),
            links: Rc::new(links),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::ipr::ToIPR;

    #[test]
    fn test_from_entries_empty() {
        let ark = Ark::from_entries::<[&str; 0]>([]);
        assert_eq!(ark.paths, vec![].into());
        assert_eq!(ark.metas, vec![].into());
        assert_eq!(ark.files, vec![].into());
        assert_eq!(ark.links, vec![].into());
    }

    #[test]
    fn test_from_entries_dirs() {
        let ark = Ark::from_entries(["foo", "bar"]);
        assert_eq!(ark.paths, vec!["bar".to_ipr(), "foo".to_ipr()].into());
        assert_eq!(ark.metas, vec![(), ()].into());
        assert_eq!(ark.files, vec![].into());
        assert_eq!(ark.links, vec![].into());
    }

    #[test]
    fn test_from_entries_uniq() {
        let ark = Ark::from_entries(["foo", "bar", "foo", "bar", "bar"]);
        assert_eq!(ark.paths, vec!["bar".to_ipr(), "foo".to_ipr()].into());
        assert_eq!(ark.metas, vec![(), ()].into());
        assert_eq!(ark.files, vec![].into());
        assert_eq!(ark.links, vec![].into());
    }

    #[test]
    fn test_from_entries_with_files() {
        let ark = Ark::from_entries([
            ("/dir1", None),
            ("/file2.txt", Some("file2 contents")),
            ("/file1.txt", Some("file1 contents")),
            ("/dir3", None),
            ("/dir2", None),
            ("/file3.txt", Some("file3 contents")),
        ]);
        assert_eq!(
            ark.paths,
            vec![
                "/file1.txt".to_ipr(),
                "/file2.txt".to_ipr(),
                "/file3.txt".to_ipr(),
                "/dir1".to_ipr(),
                "/dir2".to_ipr(),
                "/dir3".to_ipr(),
            ]
            .into()
        );
        assert_eq!(ark.metas, vec![(), (), (), (), (), ()].into());
        assert_eq!(
            ark.files,
            vec![
                "file1 contents".to_owned(),
                "file2 contents".to_owned(),
                "file3 contents".to_owned(),
            ]
            .into()
        );
        assert_eq!(ark.links, vec![].into());
    }

    #[test]
    fn test_from_entries_with_everything() {
        let ark = Ark::from_entries([
            ("aaa", "a", Content::Directory),
            ("bbb", "b", Content::Symlink("../b".into())),
            ("ccc", "c", Content::File("Sea!".into())),
        ]);
        // Category order wins
        assert_eq!(
            ark.paths,
            vec!["ccc".to_ipr(), "bbb".to_ipr(), "aaa".to_ipr(),].into()
        );
        assert_eq!(ark.metas, vec!["c", "b", "a"].into());
        assert_eq!(ark.files, vec!["Sea!".to_owned()].into());
        assert_eq!(ark.links, vec!["../b".to_owned()].into());
    }
}
