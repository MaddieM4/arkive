//! Allows for Arks of varying content types to convert to each other.
//!
//! ```
//! use arkive::*;
//!
//! let entries = vec![
//!   ("dir", Contents::Dir),
//!   ("dir/some_file.js", Contents::File("console.log('hi!')")),
//! ];
//!
//! let ark1: Ark<&str> = entries.into(); // Each file is associated with a &str
//! let ark2: Ark<String> = ark1.into();  // But now! An owned String
//! ```
use crate::types::Ark;
use std::rc::Rc;

impl<C> Ark<C> {
    /// Easy conversion by content type.
    pub fn from_translation<SRC>(src: Ark<SRC>) -> Self
    where
        C: From<SRC>,
        SRC: Clone,
    {
        let (paths, attrs, contents) = src.decompose();
        let contents: Vec<C> = (*contents).iter().map(|t| t.clone().into()).collect();
        Self(paths, attrs, Rc::new(contents))
    }
}

impl From<Ark<&str>> for Ark<Vec<u8>> {
    fn from(src: Ark<&str>) -> Self {
        Self::from_translation(src)
    }
}
impl From<Ark<&str>> for Ark<String> {
    fn from(src: Ark<&str>) -> Self {
        Self::from_translation(src)
    }
}
impl From<Ark<String>> for Ark<Vec<u8>> {
    fn from(src: Ark<String>) -> Self {
        Self::from_translation(src)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::at;
    use crate::types::{Attrs, Contents};

    #[test]
    fn translations() {
        let str_ark: Ark<&str> = vec![
            ("/a", at! { N => "1" }, Contents::File("1")),
            ("/b", at! { N => "2" }, Contents::Dir),
            ("/c", at! { N => "3" }, Contents::File("3")),
            ("/d", at! { N => "4" }, Contents::Dir),
            ("/e", at! { N => "5" }, Contents::File("5")),
            ("/f", at! { N => "6" }, Contents::Dir),
        ]
        .into();

        let string_ark: Ark<String> = str_ark.clone().into();
        let ba1: Ark<Vec<u8>> = str_ark.clone().into();
        let ba2: Ark<Vec<u8>> = string_ark.into();
        assert_eq!(ba1, ba2);
    }
}
