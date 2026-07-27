use crate::hash::{HashAlgorithm, NodeHash, Sha256Hasher};
use serde::{Deserialize, Serialize};

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
        let leaves: Vec<NodeHash> = data.iter().map(|&d| Sha256Hasher::hash_data(d)).collect();
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
                    next_level.push(Sha256Hasher::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    next_level.push(Sha256Hasher::hash_pair(&chunk[0], &chunk[0])); // duplicate final leaf for odd-width level
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

    pub fn levels_len(&self) -> usize {
        self.levels.len()
    }
}

#[cfg(test)]
mod tests {
    use super::MerkleTree;
    use crate::hash::{HashAlgorithm, Sha256Hasher};

    #[test]
    fn empty_tree_has_no_levels_and_no_root() {
        let tree = MerkleTree::new(&[]);

        assert_eq!(tree.levels_len(), 0);
        assert!(tree.root_hash().is_none());
        assert!(tree.root().is_none());
    }

    #[test]
    fn single_leaf_tree_root_matches_leaf_hash() {
        let leaf = b"alpha";
        let tree = MerkleTree::new(&[leaf.as_slice()]);
        let expected = Sha256Hasher::hash_data(leaf);

        assert_eq!(tree.levels_len(), 1);
        assert_eq!(tree.root_hash(), Some(expected.clone()));
        assert_eq!(tree.root(), Some(expected.to_vec()));
    }

    #[test]
    fn even_leaf_count_hashes_adjacent_pairs() {
        let a = Sha256Hasher::hash_data(b"a");
        let b = Sha256Hasher::hash_data(b"b");

        let tree = MerkleTree::from_hashed_leaves(vec![a.clone(), b.clone()]);
        let expected_root = Sha256Hasher::hash_pair(&a, &b);

        assert_eq!(tree.levels_len(), 2);
        assert_eq!(tree.root_hash(), Some(expected_root));
    }

    #[test]
    fn odd_leaf_count_duplicates_last_leaf_before_pairing() {
        let a = Sha256Hasher::hash_data(b"a");
        let b = Sha256Hasher::hash_data(b"b");
        let c = Sha256Hasher::hash_data(b"c");

        let tree = MerkleTree::from_hashed_leaves(vec![a.clone(), b.clone(), c.clone()]);

        let left = Sha256Hasher::hash_pair(&a, &b);
        let right = Sha256Hasher::hash_pair(&c, &c);
        let expected_root = Sha256Hasher::hash_pair(&left, &right);

        assert_eq!(tree.levels_len(), 3);
        assert_eq!(tree.root_hash(), Some(expected_root));
    }
}
