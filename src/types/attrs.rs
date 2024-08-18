//! Potentially arbitrary attributes on files or directories.
//!
//! Specific contexts might have specific expectations about what
//! attributes are present, and how to interpret them. For example,
//! UNIX permissions and uid/gid. There's really just two rules:
//!
//!  1. It's always valid to omit any/all attributes. Some reasonable
//!     and intuitive behavior ought to ensue.
//!  2. Endeavor to tag files and directories with accurate Attrs.
//!
//! ```
//! use arkive::types::attrs::*;
//!
//! let attrs = Attrs::new()
//!   .set("UNIX_UID", "1000")
//!   .append("X-SOME-ARBITRARY-THING", "Yo!");
//!
//! assert_eq!(attrs.items()[0].name(), "UNIX_UID");
//! ```

use serde::{Deserialize, Serialize};
use std::os::unix::fs::PermissionsExt;

/// A single attribute on a file or directory.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Attr(String, String);
impl Attr {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self(name.into(), value.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
    pub fn value(&self) -> &str {
        &self.1
    }
}

/// All attributes on a file or directory.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Attrs(Vec<Attr>);
impl Attrs {
    pub fn new() -> Self {
        Self(vec![])
    }

    /// Append an Attr to the list.
    ///
    /// This can be redundant with existing attrs of the same name.
    pub fn append(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.push(Attr::new(name, value));
        self
    }

    /// Delete every attr with a given name.
    pub fn delete(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.0.retain(|attr| attr.name() != name);
        self
    }

    /// Set a value in an Attrs list.
    ///
    /// It's possible to have an attribute name show up multiple times in a single
    /// Attrs object. That's valid! But sometimes you really do want to overwrite
    /// existing values, as if you were setting a value in a HashMap, and that's
    /// what `set()` does - it's literally just shorthand for `.delete(k).append(k,v)`.
    pub fn set(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        self.delete(&name).append(name, value)
    }

    /// Borrow underlying Vec.
    pub fn items(&self) -> &Vec<Attr> {
        &self.0
    }
}

impl From<std::fs::Metadata> for Attrs {
    fn from(meta: std::fs::Metadata) -> Attrs {
        Attrs::new().append("UNIX_MODE", meta.permissions().mode().to_string())
    }
}

/// A macro that makes it easy to construct Attrs objects.
///
/// If you look through the code, we make pervasive use of this macro, because
/// it's so much more concise and readable, which leads to less human error.
///
/// ```
/// use arkive::{at, Attrs};
///
/// assert_eq!(
///     at! { HELLO => "WORLD", GOOD => "JOB" },
///     Attrs::new().append("HELLO","WORLD").append("GOOD","JOB")
/// );
/// ```
#[macro_export]
macro_rules! at {
    ( $( $k:expr => $v:expr ),* ) => {
        {
            Attrs::new() $( .append(stringify!($k), $v) )*
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn attr_new() {
        let attr = Attr::new("foo", "bar");
        assert_eq!(attr.name(), "foo");
        assert_eq!(attr.value(), "bar");
    }

    #[test]
    fn attr_json_serialize() {
        let attr = Attr::new("foo", "bar");
        assert_eq!(
            serde_json::to_string(&attr).expect("should serialize"),
            "[\"foo\",\"bar\"]"
        );
    }

    #[test]
    fn attr_json_deserialize() {
        assert_eq!(
            Attr::new("N", "V"),
            serde_json::from_str("[\"N\",\"V\"]").expect("should deserialize")
        );
    }

    #[test]
    fn attrs_append() {
        let attrs = Attrs::new()
            .append("FIRST", "1")
            .append("SECOND", "2")
            .append("THIRD", "3");
        assert_eq!(attrs.0[2], Attr::new("THIRD", "3"));
    }

    #[test]
    fn at_macro() {
        let attrs = Attrs::new()
            .append("A", "1")
            .append("B", "2")
            .append("C", "3")
            .append("B", "4");
        assert_eq!(at! { A=>"1", B=>"2", C=>"3", B=>"4" }, attrs);
    }

    #[test]
    fn attrs_delete() {
        let attrs = Attrs::new()
            .append("FIRST", "1")
            .append("SECOND", "2")
            .append("THIRD", "3")
            .delete("SECOND");
        assert_eq!(
            attrs,
            Attrs::new().append("FIRST", "1").append("THIRD", "3")
        );
    }

    #[test]
    fn attrs_set() {
        let attrs = Attrs::new()
            .set("FIRST", "(hehe, first!)")
            .set("OVERWRITE_ME", "value you'll never see")
            .set("SOMETHING_ELSE", "take up some more space")
            .set("OVERWRITE_ME", "value you WILL see");
        assert_eq!(
            attrs,
            Attrs::new()
                .append("FIRST", "(hehe, first!)")
                .append("SOMETHING_ELSE", "take up some more space")
                .append("OVERWRITE_ME", "value you WILL see")
        );
    }
}
