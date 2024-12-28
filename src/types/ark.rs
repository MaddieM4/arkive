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
    pub fn from_entries<SRC>(src: SRC) -> Self
    where
        SRC: IntoIterator,
        SRC::Item: ToEntry<Content = C, Metadata = M>,
    {
        let uniq: HashMap<IPR, (M, Content<C>)> =
            to_entries(src).map(|(p, m, c)| (p, (m, c))).collect();

        let mut entries: Vec<Entry<C, M>> = uniq.into_iter().map(|(p, (m, c))| (p, m, c)).collect();

        // TODO: Sort by type too
        // (final section depends on it)
        entries.sort_unstable_by(|a, b| a.0.cmp(&b.0));

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
}
