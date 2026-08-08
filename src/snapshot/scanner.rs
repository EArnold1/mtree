use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    MerkleTree, NodeHash,
    error::MtreeError,
    hash::hash_data,
    snapshot::{
        FileEntry,
        encoding::{hash_directory_node, hash_file_node},
    },
    utils::file::relative_path,
};

pub fn walk_directory(
    root: &Path,
    current: &Path,
    directories: &mut Vec<PathBuf>,
    files: &mut Vec<FileEntry>,
) -> Result<Option<NodeHash>, MtreeError> {
    let mut entries: Vec<_> = fs::read_dir(current)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut child_hashes = Vec::new();

    for entry in entries {
        let path = entry.path();
        let entry_type = entry.file_type()?;
        let relative = relative_path(root, &path)?;

        if entry_type.is_symlink() {
            return Err(MtreeError::UnsupportedEntry(path));
        }

        if entry_type.is_dir() {
            // Scan first so directories that contain only empty directories are skipped too.
            // Keeping child entries local prevents a skipped subtree from being recorded.
            let mut child_directories = Vec::new();
            let mut child_files = Vec::new();
            let subtree_hash =
                walk_directory(root, &path, &mut child_directories, &mut child_files)?;

            if let Some(subtree_hash) = subtree_hash {
                directories.push(relative.clone());
                directories.extend(child_directories);
                files.extend(child_files);
                child_hashes.push(hash_directory_node(&relative, subtree_hash));
            }
            continue;
        }

        if entry_type.is_file() {
            let contents = fs::read(&path)?; // TODO: read in chunks
            let metadata = entry.metadata()?;
            let file = FileEntry {
                path: relative,
                hash: hash_data(&contents),
                size: metadata.len(),
            };
            let file_hash = hash_file_node(&file);
            files.push(file);
            child_hashes.push(file_hash);
            continue;
        }

        return Err(MtreeError::UnsupportedEntry(path));
    }

    if child_hashes.is_empty() {
        Ok(None)
    } else {
        let root_hash = MerkleTree::from_hashed_leaves(child_hashes)
            .root_hash()
            .expect("non-empty child hash list should always produce a Merkle root");
        Ok(Some(root_hash))
    }
}
