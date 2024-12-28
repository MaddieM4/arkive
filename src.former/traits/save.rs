//! Saves this Ark to the DB. Only works if the content is serializable.
use crate::types::*;
use std::io::Result;

pub trait Save: ToJson {
    fn save(&self, db: &DB) -> Result<Digest> {
        let json = self.to_json()?;
        let d = json.to_digest();
        std::fs::write(db.join("cas").join(d.to_hex()), json)?;
        Ok(d)
    }
}

pub trait ToJson: serde::Serialize {
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

impl<C> Save for Ark<C> where Ark<C>: ToJson {}
impl<C> ToJson for Ark<C> where Ark<C>: serde::Serialize {}

pub trait ToDigest {
    fn to_digest(&self) -> Digest;
}

impl ToDigest for String {
    fn to_digest(&self) -> Digest {
        Digest::from(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn save() -> Result<()> {
        let db = DB::new_temp()?;
        let digest = Ark::from_entries([("/hello", Contents::File("world"))]).save(&db)?;
        assert_eq!(
            digest.to_hex(),
            "e1bf5a0db81cf1bdbe24152057618a2038395e792dc27eb5cd045415588c701e"
        );

        Ok(())
    }
}
