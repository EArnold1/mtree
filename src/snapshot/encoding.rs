use std::path::Path;

use crate::{
    NodeHash,
    hash::hash_data,
    hash::payload::{Payload, PayloadType},
    snapshot::FileEntry,
    utils::file::normalize_path,
};

pub fn hash_directory_node(path: &Path, subtree_hash: NodeHash) -> NodeHash {
    let payload = Payload::new(
        PayloadType::Directory,
        &normalize_path(path),
        &subtree_hash.to_vec(),
    )
    .to_bytes();
    hash_data(&payload)
}

pub fn hash_file_node(file: &FileEntry) -> NodeHash {
    let payload = Payload::new(
        PayloadType::File,
        &normalize_path(&file.path),
        &file.content_hash.to_vec(),
    )
    .to_bytes();
    hash_data(&payload)
}
