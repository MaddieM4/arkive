//! Convert to and from lists of entries.
//!
//! These are just vectors of tuples, in either of these forms:
//!
//!  * `(path, attrs, contents)`
//!  * `(path, contents)`
//!
//! The latter is more useful for simple demonstrations, but in real world use
//! cases, attrs store information you usually care about later, like the
//! permissions (including executable bit) of files and directories.
//!
//! ```
//! use arkive::*;
//!
//! // Infer blank attrs for each.
//! let ark = Ark::from_entries([
//!     ("dir", Contents::Dir),
//!     ("dir/some_file.js", Contents::File("console.log('hi!')")),
//! ]);
//!
//! // Produces a (p,a,c)-style vector.
//! //
//! // Note that the order is different! Arks have a canon order for storing
//! // their records. The original order is not preserved.
//! let entries = ark.to_entries();
//! assert_eq!(entries[0],
//!     ("dir/some_file.js".to_ipr(), Attrs::new(), Contents::File("console.log('hi!')")));
//! ```
use crate::types::*;
use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;

/// Internal conversion format.
pub struct Entry<C>(IPR, Attrs, Contents<C>);

impl<S, C> From<(S, Attrs, Contents<C>)> for Entry<C>
where
    S: Into<IPR>,
{
    fn from(src: (S, Attrs, Contents<C>)) -> Self {
        Self(src.0.into(), src.1, src.2)
    }
}

impl<S, C> From<(S, Contents<C>)> for Entry<C>
where
    S: Into<IPR>,
{
    fn from(src: (S, Contents<C>)) -> Self {
        Self(src.0.into(), Attrs::new(), src.1)
    }
}

impl<C> Ark<C> {
    /// Read entries into Ark format.
    pub fn from_entries<E>(entries: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<Entry<C>>,
    {
        let uniq: HashMap<IPR, (Attrs, Contents<C>)> = entries
            .into_iter()
            .map(|e| {
                let entry: Entry<C> = e.into();
                (entry.0, (entry.1, entry.2))
            })
            .collect();

        let (mut files, mut dirs): (Vec<_>, Vec<_>) = uniq
            .into_iter()
            .map(|(p, (a, c))| (p, a, c))
            .partition(|(_, _, c)| c.is_file());

        // Let's start putting stuff in boxes here.
        let n = files.len() + dirs.len();
        let mut paths = Vec::<IPR>::with_capacity(n);
        let mut attrs = Vec::<Attrs>::with_capacity(n);
        let mut contents = Vec::<C>::with_capacity(files.len());

        files.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        for (p, a, c) in files {
            paths.push(p);
            attrs.push(a);
            if let Contents::File(content) = c {
                contents.push(content)
            }
        }

        dirs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        for (p, a, _) in dirs {
            paths.push(p);
            attrs.push(a);
        }

        Self(Rc::new(paths), Rc::new(attrs), Rc::new(contents))
    }

    /// Turn this Ark into a Vec of entries.
    pub fn to_entries(self) -> Vec<(IPR, Attrs, Contents<C>)>
    where
        Vec<C>: Clone,
    {
        let (paths, attrs, contents) = self.decompose();
        let file_contents = (*contents).clone().into_iter().map(|c| Contents::File(c));
        let dir_contents = std::iter::from_fn(move || Some(Contents::Dir));
        let contents = file_contents.chain(dir_contents);

        zip((*paths).clone(), (*attrs).clone())
            .zip(contents)
            .map(|((p, a), c)| (p, a, c))
            .collect()
    }
}

impl<C, S> From<Vec<(S, Attrs, Contents<C>)>> for Ark<C>
where
    S: Into<IPR>,
{
    fn from(src: Vec<(S, Attrs, Contents<C>)>) -> Self {
        Ark::from_entries(src)
    }
}

impl<C, S> From<Vec<(S, Contents<C>)>> for Ark<C>
where
    S: Into<IPR>,
{
    fn from(src: Vec<(S, Contents<C>)>) -> Self {
        src.into_iter()
            .map(|(p, c)| (p, Attrs::new(), c))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<C> From<Ark<C>> for Vec<(IPR, Attrs, Contents<C>)>
where
    Vec<C>: Clone,
{
    fn from(src: Ark<C>) -> Self {
        let (paths, attrs, contents) = src.decompose();
        let file_contents = (*contents).clone().into_iter().map(|c| Contents::File(c));
        let dir_contents = std::iter::from_fn(move || Some(Contents::Dir));
        let contents = file_contents.chain(dir_contents);

        zip((*paths).clone(), (*attrs).clone())
            .zip(contents)
            .map(|((p, a), c)| (p, a, c))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;

    #[test]
    fn empty() {
        // FROM
        let entries: Vec<(&str, Attrs, Contents<()>)> = vec![];
        let ark: Ark<()> = entries.into();
        assert_eq!(ark.paths(), &vec![] as &Vec::<IPR>);
        assert_eq!(ark.attrs(), &vec![] as &Vec::<Attrs>);
        assert_eq!(ark.contents(), &vec![] as &Vec::<()>);

        // TO
        let entries: Vec<(IPR, Attrs, Contents<()>)> = ark.into();
        assert_eq!(entries, vec![]);
    }

    #[test]
    fn one_dir() {
        // FROM
        let ark: Ark<&'static str> = vec![("/hello", at! {HELLO => "world"}, Contents::Dir)].into();
        assert_eq!(ark.paths(), &vec!["/hello"]);
        assert_eq!(ark.attrs(), &vec![at! {HELLO => "world"}]);
        assert_eq!(ark.contents(), &vec![] as &Vec::<&'static str>);

        // TO
        let entries: Vec<(IPR, Attrs, Contents<&str>)> = ark.into();
        assert_eq!(
            entries,
            vec![("/hello".into(), at! {HELLO=>"world"}, Contents::Dir)]
        );
    }

    #[test]
    fn one_file() {
        // FROM
        let ark: Ark<_> = vec![(
            "/hello.txt",
            at! {HELLO => "with text"},
            Contents::File("Some contents"),
        )]
        .into();
        assert_eq!(ark.paths(), &vec!["/hello.txt"]);
        assert_eq!(ark.attrs(), &vec![at! {HELLO => "with text"}]);
        assert_eq!(ark.contents(), &vec!["Some contents"]);

        // TO
        let entries: Vec<(IPR, Attrs, Contents<&str>)> = ark.into();
        assert_eq!(
            entries,
            vec![(
                "/hello.txt".into(),
                at! {HELLO=>"with text"},
                Contents::File("Some contents")
            )]
        );
    }

    #[test]
    fn mix() {
        // FROM
        let ark: Ark<_> = vec![
            (
                "/hello.txt",
                at! {HELLO => "with text"},
                Contents::File("Some contents"),
            ),
            ("/another", at! { DIR => "yeah" }, Contents::Dir),
            (
                "/another/file.txt",
                at! { ANOTHER => "file" },
                Contents::File("Different contents"),
            ),
        ]
        .into();

        // Files before dirs, each sorted
        assert_eq!(
            ark.paths(),
            &vec!["/another/file.txt", "/hello.txt", "/another"]
        );

        // Match order
        assert_eq!(
            ark.attrs(),
            &vec![
                at! {ANOTHER => "file"},
                at! {HELLO => "with text"},
                at! {DIR => "yeah"},
            ]
        );
        assert_eq!(
            ark.contents(),
            &vec!["Different contents", "Some contents",]
        );

        // TO
        let entries: Vec<(IPR, Attrs, Contents<&str>)> = ark.into();
        assert_eq!(
            entries,
            vec![
                (
                    "/another/file.txt".into(),
                    at! { ANOTHER => "file" },
                    Contents::File("Different contents"),
                ),
                (
                    "/hello.txt".into(),
                    at! {HELLO => "with text"},
                    Contents::File("Some contents"),
                ),
                ("/another".into(), at! { DIR => "yeah" }, Contents::Dir),
            ]
        );
    }

    #[test]
    fn overrides() {
        // FROM
        let ark: Ark<_> = vec![
            ("/x", at! { N => "1"}, Contents::File("1")),
            ("/x", at! { N => "2" }, Contents::Dir),
            ("/x", at! { N => "3"}, Contents::File("3")),
            ("/x", at! { N => "4" }, Contents::Dir),
            ("/x", at! { N => "5"}, Contents::File("5")),
            ("/x", at! { N => "6" }, Contents::Dir),
        ]
        .into();

        // Last item should win
        assert_eq!(ark.paths(), &vec!["/x"]);
        assert_eq!(ark.attrs(), &vec![at! { N => "6"}]);
        assert_eq!(ark.contents(), &vec![] as &Vec::<&str>);

        // TO
        let entries: Vec<(IPR, Attrs, Contents<&str>)> = ark.into();
        assert_eq!(entries, vec![("/x".into(), at! {N => "6"}, Contents::Dir)]);
    }
}
