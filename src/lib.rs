//! A library for efficient bulk operations on immutable directory data.
//!
//! The filesystem isn't immutable, in fact it's very, very mutable. So it's a
//! little tricky to do operations on the filesystem "as if" they're immutable.
//! But if you could, there's lots of reasons that could be useful. You'd have a
//! "save point" at every little step of the process. You could cache any
//! actions you take that are _deterministic_ (always get the same results when
//! you do the same thing), letting you skip work that you've done before. And
//! there's things you could do exceptionally fast by doing most of the work in
//! memory.
//!
//! That's what Dirtabase does, although it handles more the "build pipelines"
//! part of the problem. The underlying Swiss Army knife for Dirtabase is Ark,
//! which is a data structure that lets you do a lot of work in-memory,
//! efficiently. Check out the [`crate::types::ark::Ark`] type for more detail.
//!
//! tl;dr It's like Virtual DOM but applied to directory data.
//!
//! ```
//! use arkive::*;
//!
//! // You can associate ANYTHING with files in an Ark as their content type.
//! //
//! // Seems kinda goofy to lead with a "what about integers" example, but
//! // that freedom leads to good questions like, "what if those were file
//! // sizes? And I was building a tool to free up disk space?"
//! let ark = Ark::from_entries([
//!     ("a/path", Contents::Dir),
//!     ("a/path/to/a/file", Contents::File(34)),
//!     ("another/file", Contents::File(910)),
//! ]);
//!
//! assert_eq!(ark.len(), 3);
//! ```

pub mod traits;
pub mod types;

pub use traits::*;
pub use types::*;

#[cfg(test)]
static TEST_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(test)]
fn setup_fixture() {
    TEST_INIT.call_once(|| {
        std::process::Command::new("chmod")
            .args(["-R", "g+w", "fixture"])
            .status()
            .expect("Failed to fix fixture permissions");
    });
}
