use std::path::Path;

use crate::{
    NodeHash,
    hash::hash_data,
    hash::payload::{Payload, PayloadType},
    snapshot::FileEntry,
    utils::file::normalize_path,
};

pub fn hash_directory_node(path: &Path, subtree_hash: Option<NodeHash>) -> NodeHash {
    let payload = Payload::new(
        PayloadType::Directory,
        &normalize_path(path),
        subtree_hash.map(|h| h.to_vec()).as_deref(),
    )
    .to_bytes();
    hash_data(&payload)
}

pub fn hash_file_node(file: &FileEntry) -> NodeHash {
    let payload = Payload::new(
        PayloadType::File,
        &normalize_path(&file.path),
        Some(&file.hash.to_vec()),
    )
    .to_bytes();
    hash_data(&payload)
}
