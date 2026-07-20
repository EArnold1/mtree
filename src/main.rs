use mtree::MerkleTree;

fn main() {
    let data = vec![
        b"transaction 1".as_slice(),
        b"transaction 2".as_slice(),
        b"transaction 3".as_slice(),
        b"transaction 4".as_slice(),
        b"transaction 5".as_slice(), // 5 items will test the odd-node logic
    ];

    let tree = MerkleTree::new(&data);

    if let Some(root) = tree.root() {
        // Print the root hash as a hex string
        let hex_root: String = root.iter().map(|b| format!("{:02x}", b)).collect();
        println!("Merkle Root: {}", hex_root);
        println!("Total levels: {}", tree.levels_len());
    }
}
