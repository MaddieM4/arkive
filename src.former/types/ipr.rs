//! Internal path represenation. UNIX paths, but stricter.

use lazy_regex::regex;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Internal Path Representation.
///
/// They obey the following rules:
///
///  * Valid UTF-8
///  * Are path-separated with /
///  * Do not begin or end with /
///  * Do not contain . or .. segments
///  * Do not contain multiple / characters right next to each other
///
/// We can actually infallibly convert to these constraints as long as the input
/// we're working with is already UTF-8, which Rust strings are! It's worthwhile
/// to note that this is actually much pickier in certain ways than actual UNIX
/// paths, and you may find some imports fail when dealing with non-unicode
/// paths. The official dirtabase/ark policy for these situations is "we aren't
/// making stuff for your use case."
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize)]
pub struct IPR(String);

impl AsRef<str> for IPR {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T> PartialEq<T> for IPR
where
    T: ToIPR,
{
    fn eq(&self, other: &T) -> bool {
        self.eq(&other.to_ipr())
    }
}

/// Types that can be converted to IPRs.
pub trait ToIPR: AsRef<str> {
    fn to_ipr(&self) -> IPR {
        let converted = IPR::canonize(self.as_ref());
        IPR(converted.to_string())
    }
}

// This makes the From implementatsions so much easier. Seriously.
impl ToIPR for &str {}
impl ToIPR for &&str {}
impl ToIPR for String {}
impl ToIPR for &String {}

impl From<&str> for IPR {
    fn from(other: &str) -> IPR {
        other.to_ipr()
    }
}
impl From<&&str> for IPR {
    fn from(other: &&str) -> IPR {
        other.to_ipr()
    }
}
impl From<String> for IPR {
    fn from(other: String) -> IPR {
        other.to_ipr()
    }
}
impl From<&String> for IPR {
    fn from(other: &String) -> IPR {
        other.to_ipr()
    }
}

/// Conversion failed because path wasn't valid Unicode.
///
/// That's not required for operating systems! But it's required for Ark. Life
/// is just really hard in a lot of ways down the road if you don't have this
/// baseline of sanity established.
#[derive(Debug)]
pub struct NonUnicodePath(pub PathBuf);
impl std::fmt::Display for NonUnicodePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Path wasn't unicode: {:?}", self.0)
    }
}
impl std::error::Error for NonUnicodePath {}
impl From<NonUnicodePath> for std::io::Error {
    fn from(p: NonUnicodePath) -> Self {
        Self::other(p)
    }
}

impl TryFrom<&Path> for IPR {
    type Error = NonUnicodePath;
    fn try_from(other: &Path) -> Result<IPR, Self::Error> {
        Ok(other
            .to_str()
            .ok_or_else(|| NonUnicodePath(other.to_path_buf()))?
            .into())
    }
}
impl TryFrom<PathBuf> for IPR {
    type Error = NonUnicodePath;
    fn try_from(other: PathBuf) -> Result<IPR, Self::Error> {
        let p: &Path = other.as_ref();
        Self::try_from(p)
    }
}

// Deserialization done manually in order to not trust loaded arks.
//
// This is a pain, but it gets a little more manageable each time I have to
// write one of these. I think it comes from the weird little gap of the process
// that you're filling when you write a deserializer. Probably makes a lot of
// sense in full context.
struct IPRVisitor;
impl<'de> serde::de::Visitor<'de> for IPRVisitor {
    type Value = IPR;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a path string (IPR)")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value.to_ipr())
    }
}

impl<'de> Deserialize<'de> for IPR {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(IPRVisitor)
    }
}

impl IPR {
    /// Quick check to see if a string already meets requirements.
    pub fn is_well_formed(src: &str) -> bool {
        // Quick and nasty way of avoiding regex cost.
        // Probably should have benched this before all the work.
        // I am not immune to premature optimization.
        enum SM {
            Strt,
            Slsh,
            Dot1,
            Dot2,
            Char,
        }

        let mut sm = SM::Strt;
        for c in src.chars() {
            // nc = next character
            sm = match (sm, c) {
                (SM::Strt, '/') => return false, // No leading slashes
                (SM::Strt, '.') => SM::Dot1,
                (SM::Strt, ___) => SM::Char,
                (SM::Slsh, '/') => return false, // Invalid: "//"
                (SM::Slsh, '.') => SM::Dot1,
                (SM::Slsh, ___) => SM::Char,
                (SM::Dot1, '/') => return false, // Invalid: "./"
                (SM::Dot1, '.') => SM::Dot2,
                (SM::Dot1, ___) => SM::Char,
                (SM::Dot2, '/') => return false, // Invalid: "../"
                (SM::Dot2, '.') => SM::Char,     // "..." is actually okay
                (SM::Dot2, ___) => SM::Char,
                (SM::Char, '/') => SM::Slsh,
                (SM::Char, '.') => SM::Char,
                (SM::Char, ___) => SM::Char,
            }
        }
        // What are valid final characters?
        match sm {
            SM::Strt => true,
            SM::Slsh => false,
            SM::Dot1 => false,
            SM::Dot2 => false,
            SM::Char => true,
        }
    }

    /// Produce a [`&str`] in IPR form.
    ///
    /// Reuses original string if it's already suitable. Hence the Cow.
    pub fn canonize<'a>(src: &'a str) -> Cow<'a, str> {
        if IPR::is_well_formed(&src) {
            Cow::Borrowed(src)
        } else {
            Cow::Owned(Self::force_canonize(&src))
        }
    }

    /// Produce an owned [`String`] in IPR form.
    ///
    /// Doesn't short-circuit to reuse existing memory. Just always charges
    /// forward with a statically compiled regex and gets 'er done.
    pub fn force_canonize(src: &str) -> String {
        let r = regex!("/");
        r.split(src)
            .filter(|s| *s != "" && *s != "." && *s != "..")
            .collect::<Vec<&str>>()
            .join("/")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_well_formed() {
        fn check(case: &str, expected: bool) {
            assert_eq!(IPR::is_well_formed(case), expected, "Failed on: {:?}", case);
        }
        check("", true);
        check("foo", true);
        check("foo/bar", true);
        check("/foo/bar", false);
        check("/foo/bar/", false);
        check("foo/bar////", false);
        check("////foo////bar", false);
        check(".", false);
        check("..", false);
        check("...", true);
        check("....", true);
        check(".....", true);
        check("foo.", true);
        check("a/./b", false);
        check("a/../b", false);
        check("a/.../b", true);
        check("a/..../b", true);
        check("a/...../b", true);
    }

    #[test]
    fn canonize() {
        fn check(case: &str, expected: &str) {
            assert_eq!(
                IPR::canonize(case),
                expected,
                "IPR::canonize failed on: {:?}",
                case
            );
            assert_eq!(
                IPR::force_canonize(case),
                expected,
                "IPR::force_canonize failed on: {:?}",
                case
            );
        }
        check("", "");
        check("foo", "foo");
        check("foo/bar", "foo/bar");
        check("/foo/bar", "foo/bar");
        check("/foo/bar/", "foo/bar");
        check("foo/bar////", "foo/bar");
        check("////foo////bar", "foo/bar");
        check(".", "");
        check("..", "");
        check("...", "...");
        check("....", "....");
        check(".....", ".....");
        check("foo.", "foo.");
        check("a/./b", "a/b");
        check("a/../b", "a/b");
        check("a/.../b", "a/.../b");
        check("a/..../b", "a/..../b");
        check("a/...../b", "a/...../b");
    }

    #[test]
    fn convert_str() {
        let original: &str = "/hello/world/";
        let converted: IPR = original.into();
        assert_eq!(converted.0, "hello/world".to_owned());
    }

    #[test]
    fn convert_string() {
        let original: String = "/hello/world/".to_owned();
        let converted: IPR = original.into();
        assert_eq!(converted.0, "hello/world".to_owned());
    }

    #[test]
    fn convert_path() {
        let original: &Path = Path::new("/foo/bar//baz");
        let converted: IPR = original.try_into().expect("That's basically sane");
        assert_eq!(converted.0, "foo/bar/baz");
    }

    #[test]
    fn convert_pathbuf() {
        let original: PathBuf = Path::new("/foo/bar//baz").to_path_buf();
        let converted: IPR = original.try_into().expect("That's basically sane");
        assert_eq!(converted.0, "foo/bar/baz");
    }

    #[test]
    fn serialize() {
        let some_path: IPR = "/fixme/".into();
        let serialized = serde_json::to_string(&some_path).expect("Serialized OK");
        assert_eq!(serialized, "\"fixme\"");
    }

    #[test]
    fn deserialize() {
        let serialized = "\"/fixme/\""; // Has issues!
        let deserialized: IPR = serde_json::from_str(&serialized).expect("Deserialized OK");
        assert_eq!(deserialized.0, "fixme".to_owned());
    }
}
