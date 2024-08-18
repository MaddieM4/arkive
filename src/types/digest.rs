//! Compute cryptographic hash digests of content, in a way where you'd
//! have to go very explicitly out of your way to fake a hash or get it
//! wrong.
//!
//! ```
//! use arkive::Digest;
//!
//! let d: Digest = "foo".into();
//! println!("{}", d.to_hex());
//! ```

// Don't pollute namespace, just make sure we're loading some traits
use hex::ToHex;
use serde::de;
use serde::de::Visitor;
use sha2::Digest as _;

/// Trait for upstream vendor tools which produce digests.
pub trait Hasher<const N: usize>: sha2::Digest + Sized {
    /// Produce an N-byte digest as a raw byte array.
    fn into_bytes(self) -> [u8; N];

    /// Produce a Dirtabase digest type.
    fn into_digest(self) -> D<N> {
        D::<N>(self.into_bytes())
    }
}

// Need one of these per algorithm
impl Hasher<{ 256 / 8 }> for sha2::Sha256 {
    fn into_bytes(self) -> [u8; 256 / 8] {
        self.finalize().as_slice().try_into().unwrap()
    }
}
impl Hasher<{ 512 / 8 }> for sha2::Sha512 {
    fn into_bytes(self) -> [u8; 512 / 8] {
        self.finalize().as_slice().try_into().unwrap()
    }
}

/// Generic digest implementation.
///
/// This is used to make the "real" types you'd use every day, particularly
/// Digest. This flexibility should be somewhat helpful if Sha256 ever proves
/// inadequate, which isn't likely in the _near_ future, but is plausible on a
/// long enough timescale.
#[derive(PartialEq, Copy, Clone)]
pub struct D<const N: usize>([u8; N]);
impl<const N: usize> D<N> {
    /// Machine-friendly borrow of digest bytes.
    pub fn to_bytes(&self) -> &[u8; N] {
        &self.0
    }

    /// Import from some external source. Be careful to preserve invariants!
    pub fn from_bytes(bytes: &[u8; N]) -> Self {
        Self(bytes.clone())
    }

    /// Human-friendly representation of digest bytes.
    pub fn to_hex(&self) -> String {
        self.to_bytes().encode_hex()
    }

    /// Create a digest from a text hex
    pub fn from_hex(h: impl AsRef<[u8]>) -> Result<Self, hex::FromHexError> {
        let mut bytes = [0u8; N];
        hex::decode_to_slice(h, &mut bytes)?;
        Ok(Self(bytes))
    }
}

impl<const N: usize> std::fmt::Debug for D<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Digest({:?})", self.to_hex())
    }
}
impl<const N: usize> serde::Serialize for D<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}
impl<'de, const N: usize> serde::Deserialize<'de> for D<N> {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        struct HexVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for HexVisitor<N> {
            type Value = D<N>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a hex string representing {} bytes", N)
            }

            fn visit_str<E>(self, value: &str) -> Result<D<N>, E>
            where
                E: de::Error,
            {
                let vec = hex::decode(value).map_err(de::Error::custom)?;
                let bytes: [u8; N] = vec
                    .try_into()
                    .map_err(|o: Vec<u8>| de::Error::invalid_length(o.len(), &self))?;
                Ok(D::from_bytes(&bytes))
            }
        }

        deserializer.deserialize_str(HexVisitor::<N>)
    }
}

// We divide by 8 since these are named after the number of bits, not bytes.
pub type DigestSha256 = D<{ 256 / 8 }>;
pub type DigestSha512 = D<{ 512 / 8 }>;

/// The default Digest type used throughout Dirtabase.
pub type Digest = DigestSha256;

impl<T> From<T> for DigestSha256
where
    T: AsRef<[u8]>,
{
    fn from(data: T) -> Self {
        sha2::Sha256::new().chain_update(data).into_digest()
    }
}
impl<T> From<T> for DigestSha512
where
    T: AsRef<[u8]>,
{
    fn from(data: T) -> Self {
        sha2::Sha512::new().chain_update(data).into_digest()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_demo() {
        let hasher = sha2::Sha256::new();
        let d = hasher.chain_update("Hello world!").into_digest();
        assert_eq!(
            d.to_hex(),
            "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a"
        );
    }

    #[test]
    fn partial_eq() {
        let d1: Digest = "Hello from D1!".into();
        let d2: Digest = "Hello from D2!".into();
        assert_eq!(d1, d1);
        assert!(d1 != d2);
    }

    #[test]
    fn into() {
        let d: Digest = "Hello world!".into();
        assert_eq!(
            d.to_hex(),
            "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a"
        );
    }

    #[test]
    fn serialize() {
        let d: Digest = "Hello world!".into();
        let s = serde_json::to_string(&d).expect("failed to serialize");
        assert_eq!(
            s,
            "\"c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a\""
        );
    }
    #[test]
    fn deserialize() {
        let s = "\"c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a\"";
        let d: Digest = serde_json::from_str(&s).expect("failed to deserialize");
        assert_eq!(d, Digest::from("Hello world!"));

        // Error scenario 1: not a JSON string
        let d = serde_json::from_str::<Digest>("123");
        assert!(d.is_err());

        // Error scenario 2: it's not hex
        let d = serde_json::from_str::<Digest>("\"text but not hex\"");
        assert!(d.is_err());

        // Error scenario 3: Wrong length
        let d = serde_json::from_str::<Digest>("\"123\"");
        assert!(d.is_err());
    }

    #[test]
    fn from_sha256() {
        let d = Digest::from("Hello world!");
        assert_eq!(
            &d.to_hex(),
            "c0535e4be2b79ffd93291305436bf889314e4a3faec05ecffcbb7df31ad9e51a"
        );
        assert_eq!(d.to_bytes()[0..3], [192, 83, 94]);
    }

    #[test]
    fn from_sha512() {
        let d = DigestSha512::from("Hello world!");
        assert_eq!(
            &d.to_hex(),
            "f6cde2a0f819314cdde55fc227d8d7dae3d28cc556222a0a8ad66d91ccad4aad6094f517a2182360c9aacf6a3dc323162cb6fd8cdffedb0fe038f55e85ffb5b6"
        );
        assert_eq!(d.to_bytes()[0..3], [246, 205, 226]);
    }
}
