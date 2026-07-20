use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct MerkleTree {
    /// Stores the tree level by level.
    /// `levels[0]` contains the hashed leaves.
    /// `levels[levels.len() - 1]` contains the root.
    levels: Vec<Vec<Vec<u8>>>,
}

impl MerkleTree {
    /// Builds a new Merkle Tree from a slice of byte slices.
    pub fn new(data: &[&[u8]]) -> Self {
        if data.is_empty() {
            return MerkleTree { levels: vec![] };
        }

        // Step 1: Hash the initial data to create the leaves
        let leaves: Vec<Vec<u8>> = data.iter().map(|&d| Self::hash_data(d)).collect();
        let mut levels = vec![leaves.clone()];

        let mut current_level = leaves;

        // Step 2: Build the tree upwards until only the root remains
        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            // Process nodes in pairs
            for chunk in current_level.chunks(2) {
                if chunk.len() == 2 {
                    // println!("Hashing pair: {:?} ", chunk[0],);
                    // Hash the left and right child together
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[1]));
                } else {
                    // If there's an odd node out, hash it with itself
                    next_level.push(Self::hash_pair(&chunk[0], &chunk[0]));
                }
            }

            levels.push(next_level.clone());
            current_level = next_level;
        }

        MerkleTree { levels }
    }

    /// Returns the root hash of the tree.
    pub fn root(&self) -> Option<Vec<u8>> {
        self.levels
            .last()
            .and_then(|root_level| root_level.first().cloned())
    }

    /// Hashes raw data (leaf node).
    fn hash_data(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Hashes two child nodes together to create a parent node.
    fn hash_pair(left: &[u8], right: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }

    pub fn levels_len(&self) -> usize {
        self.levels.len()
    }
}
