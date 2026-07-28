use serde::{Deserialize, Serialize};

pub mod payload;
mod sha256;

pub use sha256::Sha256Hasher;

pub const HASH_SIZE: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeHash([u8; HASH_SIZE]);

impl NodeHash {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn from_array(bytes: [u8; HASH_SIZE]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for NodeHash {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Hashing strategy used by Merkle tree builders.
pub trait HashAlgorithm {
    fn hash_data(data: &[u8]) -> NodeHash;
    fn hash_pair(left: &NodeHash, right: &NodeHash) -> NodeHash;
}

pub fn hash_data(data: &[u8]) -> NodeHash {
    Sha256Hasher::hash_data(data)
}

pub fn hash_pair(left: &NodeHash, right: &NodeHash) -> NodeHash {
    Sha256Hasher::hash_pair(left, right)
}
