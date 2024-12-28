// TODO (not right now at all):
// Support as_ vs to_ vs into_ with correct label semantics.
// https://stackoverflow.com/questions/72996050/when-to-use-as-vs-to-vs-into-in-rust

use crate::types::ipr::*;

#[derive(PartialEq, Debug)]
pub enum Content<C> {
    File(C),
    Symlink(String),
    Directory,
}

pub type Entry<C, M = ()> = (IPR, M, Content<C>);

pub trait ToEntry {
    type Metadata;
    type Content;

    // Convenience function for generating an entry from other formats.
    fn to_entry(&self) -> Entry<Self::Content, Self::Metadata>;
}

impl ToEntry for &str {
    type Metadata = ();
    type Content = ();
    fn to_entry(&self) -> Entry<Self::Content, Self::Metadata> {
        (self.to_ipr(), (), Content::Directory)
    }
}

impl ToEntry for (&str, &str) {
    type Metadata = ();
    type Content = String;
    fn to_entry(&self) -> Entry<Self::Content, Self::Metadata> {
        (self.0.to_ipr(), (), Content::File(self.1.to_owned()))
    }
}

impl ToEntry for (&str, Option<&str>) {
    type Metadata = ();
    type Content = String;
    fn to_entry(&self) -> Entry<Self::Content, Self::Metadata> {
        let c = match self.1 {
            Some(contents) => Content::File(contents.to_owned()),
            None => Content::Directory,
        };
        (self.0.to_ipr(), (), c)
    }
}

pub fn to_entries<In, C, M>(e: In) -> impl Iterator<Item = Entry<C, M>>
where
    In: IntoIterator,
    In::Item: ToEntry<Content = C, Metadata = M>,
{
    e.into_iter().map(|item| item.to_entry())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_entry() {
        // &str
        assert_eq!("foo".to_entry(), ("foo".to_ipr(), (), Content::Directory));

        // (&str, &str)
        assert_eq!(
            ("/some/path", "Some contents").to_entry(),
            (
                "/some/path".to_ipr(),
                (),
                Content::<String>::File("Some contents".into())
            )
        );

        // (&str, Option<&str>)
        assert_eq!(
            ("/some/path", None).to_entry(),
            ("/some/path".to_ipr(), (), Content::Directory)
        );
        assert_eq!(
            ("/some/path", Some("Some contents")).to_entry(),
            (
                "/some/path".to_ipr(),
                (),
                Content::<String>::File("Some contents".into())
            )
        );
    }

    #[test]
    fn test_vec_to_entries() {
        let v = vec!["foo", "bar", "baz"];
        let entries = to_entries(v).collect::<Vec<_>>();
        assert_eq!(
            entries,
            vec![
                ("foo".to_ipr(), (), Content::Directory),
                ("bar".to_ipr(), (), Content::Directory),
                ("baz".to_ipr(), (), Content::Directory),
            ]
        );

        let v = vec![("a", None), ("b", Some("contents"))];
        let entries = to_entries(v).collect::<Vec<_>>();
        assert_eq!(
            entries,
            vec![
                ("a".to_ipr(), (), Content::Directory),
                ("b".to_ipr(), (), Content::File("contents".to_owned())),
            ]
        );
    }

    #[test]
    fn test_array_to_entries() {
        let v = ["foo", "bar", "baz"];
        let entries = to_entries(v).collect::<Vec<_>>();
        assert_eq!(
            entries,
            vec![
                ("foo".to_ipr(), (), Content::Directory),
                ("bar".to_ipr(), (), Content::Directory),
                ("baz".to_ipr(), (), Content::Directory),
            ]
        );

        let v = [("a", None), ("b", Some("contents"))];
        let entries = to_entries(v).collect::<Vec<_>>();
        assert_eq!(
            entries,
            vec![
                ("a".to_ipr(), (), Content::Directory),
                ("b".to_ipr(), (), Content::File("contents".to_owned())),
            ]
        );
    }
}
