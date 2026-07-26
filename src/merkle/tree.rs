use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
}

impl AsRef<[u8]> for NodeHash {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    /// Stores the tree level by level.
    /// `levels[0]` contains the hashed leaves.
    /// `levels[levels.len() - 1]` contains the root.
    levels: Vec<Vec<NodeHash>>,
}

impl MerkleTree {
    /// Builds a new Merkle Tree from a slice of byte slices.
    pub fn new(data: &[&[u8]]) -> Self {
        let leaves: Vec<NodeHash> = data.iter().map(|&d| Self::hash_data(d)).collect();
        Self::from_hashed_leaves(leaves)
    }

    /// Builds a Merkle tree from already-hashed leaves.
    pub fn from_hashed_leaves(leaves: Vec<NodeHash>) -> Self {
        if leaves.is_empty() {
            return MerkleTree { levels: vec![] };
        }

        let mut levels = vec![leaves.clone()];
        let mut current_level = leaves;

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[0])); // TODO: verify why this is done
                }
            }

            levels.push(next_level.clone());
            current_level = next_level;
        }

        MerkleTree { levels }
    }

    /// Returns the typed root hash of the tree.
    pub fn root_hash(&self) -> Option<NodeHash> {
        self.levels
            .last()
            .and_then(|root_level| root_level.first().cloned())
    }

    /// Returns the root hash of the tree.
    pub fn root(&self) -> Option<Vec<u8>> {
        self.root_hash().map(|hash| hash.to_vec())
    }

    pub fn hash_bytes(data: &[u8]) -> NodeHash {
        Self::hash_data(data)
    }

    /// Hashes raw data (leaf node).
    fn hash_data(data: &[u8]) -> NodeHash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let mut bytes = [0u8; HASH_SIZE];
        bytes.copy_from_slice(&digest);
        NodeHash(bytes)
    }

    /// Hashes two child nodes together to create a parent node.
    fn hash_pair(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hasher = Sha256::new();
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        let digest = hasher.finalize();
        let mut bytes = [0u8; HASH_SIZE];
        bytes.copy_from_slice(&digest);
        NodeHash(bytes)
    }

    pub fn levels_len(&self) -> usize {
        self.levels.len()
    }
}
