//! Loads an Ark from the DB.
use crate::types::*;
use serde::Deserialize;
use std::io::Result;

impl<C> Ark<C>
where
    Ark<C>: for<'de> Deserialize<'de>,
{
    /// Load any serializable Ark from the DB.
    ///
    /// You'll usually want to qualify this with a type, and that's almost
    /// always Digest specifically. For example:
    ///
    /// ```
    /// use arkive::*;
    /// let db = DB::new_temp()?;
    /// let digest = Ark::scan("fixture")?.import(&db)?;
    ///
    /// let ark: Ark<Digest> = Ark::load(&db, &digest)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn load(db: &DB, digest: &Digest) -> Result<Self> {
        let raw_bytes = std::fs::read(db.join("cas").join(digest.to_hex()))?;
        let serialized = String::from_utf8(raw_bytes).expect("UTF-8 bytes");
        Ok(serde_json::from_str(&serialized)?)
    }
}
