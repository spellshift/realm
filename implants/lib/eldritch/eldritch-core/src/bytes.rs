use alloc::vec::Vec;

/// A byte sequence type used in Eldritch function signatures.
///
/// This wraps a `Vec<u8>` and provides ergonomic conversions between
/// Rust binary data and Eldritch's native `bytes` type.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    /// Create an empty `Bytes` instance.
    pub fn new() -> Self {
        Bytes(Vec::new())
    }

    /// Copy bytes from a slice.
    pub fn copy_from_slice(b: &[u8]) -> Self {
        Bytes(b.to_vec())
    }

    /// Get the length of the byte sequence.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the byte sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert to a `Vec<u8>`, cloning the data.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }

    /// Consume this `Bytes` and return the inner `Vec<u8>`.
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes(v)
    }
}

impl From<&[u8]> for Bytes {
    fn from(b: &[u8]) -> Self {
        Bytes(b.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for Bytes {
    fn from(b: &[u8; N]) -> Self {
        Bytes(b.to_vec())
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(b: Bytes) -> Vec<u8> {
        b.0
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::ops::Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}
