use sha2::{Digest, Sha256};

use super::{HASH_SIZE, HashAlgorithm, NodeHash};

/// Default SHA-256 hash algorithm implementation.
pub struct Sha256Hasher;

impl HashAlgorithm for Sha256Hasher {
    fn hash_data(data: &[u8]) -> NodeHash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let mut bytes = [0u8; HASH_SIZE];
        bytes.copy_from_slice(&digest);
        NodeHash::from_array(bytes)
    }

    fn hash_pair(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = Sha256::new();
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        let digest = hasher.finalize();
        let mut bytes = [0u8; HASH_SIZE];
        bytes.copy_from_slice(&digest);
        NodeHash::from_array(bytes)
    }
}
