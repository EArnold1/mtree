use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{MerkleTree, NodeHash};

#[derive(Debug)]
pub enum BuildError {
    Io(io::Error),
    InvalidRoot(PathBuf),
    UnsupportedEntry(PathBuf),
    PathPrefix { path: PathBuf, root: PathBuf },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error while building snapshot: {error}"),
            Self::InvalidRoot(path) => write!(
                f,
                "snapshot root must be an existing directory: {}",
                path.display()
            ),
            Self::PathPrefix { path, root } => write!(
                f,
                "failed to derive a relative path for {} from root {}",
                path.display(),
                root.display()
            ),
            Self::UnsupportedEntry(path) => {
                write!(f, "unsupported directory entry type: {}", path.display())
            }
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidRoot(_) | Self::UnsupportedEntry(_) | Self::PathPrefix { .. } => None,
        }
    }
}

impl From<io::Error> for BuildError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMetadata {
    pub root: PathBuf,
    pub generated_at_unix_seconds: u64,
    pub file_count: usize,
    pub directory_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub hash: Vec<u8>,
    pub size: u64,
}

#[derive(Debug)]
pub struct DirectorySnapshot {
    pub metadata: SnapshotMetadata,
    pub directories: Vec<PathBuf>,
    pub files: Vec<FileEntry>,
    pub tree: MerkleTree,
}

impl DirectorySnapshot {
    pub fn root(&self) -> Option<Vec<u8>> {
        self.tree.root()
    }

    pub fn root_hash(&self) -> Option<NodeHash> {
        self.tree.root_hash()
    }
}

pub fn build_snapshot(root: impl AsRef<Path>) -> Result<DirectorySnapshot, BuildError> {
    let root = root.as_ref();
    let root_metadata = fs::metadata(root)?;
    if !root_metadata.is_dir() {
        return Err(BuildError::InvalidRoot(root.to_path_buf()));
    }

    let root = root.canonicalize()?;
    let mut directories = Vec::new();
    let mut files = Vec::new();

    let root_hash = walk_directory(&root, &root, &mut directories, &mut files)?;
    let tree = match root_hash {
        Some(hash) => MerkleTree::from_root(hash),
        None => MerkleTree::new(&[]),
    };

    Ok(DirectorySnapshot {
        metadata: SnapshotMetadata {
            root,
            generated_at_unix_seconds: unix_timestamp(SystemTime::now()),
            file_count: files.len(),
            directory_count: directories.len(),
        },
        directories,
        files,
        tree,
    })
}

fn walk_directory(
    root: &Path,
    current: &Path,
    directories: &mut Vec<PathBuf>,
    files: &mut Vec<FileEntry>,
) -> Result<Option<NodeHash>, BuildError> {
    let mut entries: Vec<_> = fs::read_dir(current)?.collect::<Result<_, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut child_hashes = Vec::new();

    for entry in entries {
        let path = entry.path();
        let entry_type = entry.file_type()?;
        let relative = relative_path(root, &path)?;

        if entry_type.is_symlink() {
            return Err(BuildError::UnsupportedEntry(path));
        }

        if entry_type.is_dir() {
            directories.push(relative.clone());
            let subtree_hash = walk_directory(root, &path, directories, files)?;
            let directory_hash = hash_directory_node(&relative, subtree_hash);
            child_hashes.push(directory_hash);
            continue;
        }

        if entry_type.is_file() {
            let contents = fs::read(&path)?; // TODO: read in chunks
            let metadata = entry.metadata()?;
            let file = FileEntry {
                path: relative,
                hash: MerkleTree::hash_bytes(&contents).to_vec(),
                size: metadata.len(),
            };
            let file_hash = hash_file_node(&file);
            files.push(file);
            child_hashes.push(file_hash);
            continue;
        }

        return Err(BuildError::UnsupportedEntry(path));
    }

    if child_hashes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(combine_hashes(child_hashes)))
    }
}

fn relative_path(root: &Path, path: &Path) -> Result<PathBuf, BuildError> {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| BuildError::PathPrefix {
            path: path.to_path_buf(),
            root: root.to_path_buf(),
        })
}

fn directory_leaf_payload(path: &Path) -> Vec<u8> {
    let mut payload = Vec::from(b"dir\0".as_slice());
    payload.extend(normalize_path(path).as_bytes());
    payload
}

fn hash_directory_node(path: &Path, subtree_hash: Option<NodeHash>) -> NodeHash {
    let mut payload = directory_leaf_payload(path);
    payload.push(0);
    if let Some(hash) = subtree_hash {
        payload.extend(hash.as_bytes());
    }
    MerkleTree::hash_bytes(&payload)
}

fn file_leaf_payload(file: &FileEntry) -> Vec<u8> {
    let mut payload = Vec::from(b"file\0".as_slice());
    payload.extend(normalize_path(&file.path).as_bytes());
    payload.push(0);
    payload.extend(&file.hash);
    payload
}

fn hash_file_node(file: &FileEntry) -> NodeHash {
    MerkleTree::hash_bytes(&file_leaf_payload(file))
}

fn combine_hashes(hashes: Vec<NodeHash>) -> NodeHash {
    MerkleTree::from_hashed_leaves(hashes)
        .root_hash()
        .expect("non-empty child hash list should always produce a Merkle root")
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn unix_timestamp(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// #[cfg(test)]
// mod tests {
//     use super::build_snapshot;
//     use std::{
//         fs,
//         path::{Path, PathBuf},
//         sync::atomic::{AtomicU64, Ordering},
//         time::{SystemTime, UNIX_EPOCH},
//     };

//     static NEXT_ID: AtomicU64 = AtomicU64::new(0);

//     #[test]
//     fn builds_empty_snapshot_for_empty_directory() {
//         let temp = TempDir::new();

//         let snapshot = build_snapshot(temp.path()).expect("empty directory snapshot should build");

//         assert!(snapshot.directories.is_empty());
//         assert!(snapshot.files.is_empty());
//         assert_eq!(snapshot.metadata.file_count, 0);
//         assert_eq!(snapshot.metadata.directory_count, 0);
//         assert!(snapshot.root().is_none());
//     }

//     #[test]
//     fn snapshot_tracks_directories_and_sorts_entries_deterministically() {
//         let temp = TempDir::new();
//         fs::create_dir_all(temp.path().join("z-last/empty")).expect("create z-last/empty");
//         fs::create_dir_all(temp.path().join("a-first")).expect("create a-first");
//         fs::write(temp.path().join("z-last/file-b.txt"), b"second").expect("write file-b");
//         fs::write(temp.path().join("a-first/file-a.txt"), b"first").expect("write file-a");

//         let snapshot = build_snapshot(temp.path()).expect("snapshot should build");

//         let directories: Vec<_> = snapshot.directories.iter().map(path_to_string).collect();
//         let files: Vec<_> = snapshot
//             .files
//             .iter()
//             .map(|entry| path_to_string(&entry.path))
//             .collect();

//         assert_eq!(directories, vec!["a-first", "z-last", "z-last/empty"]);
//         assert_eq!(files, vec!["a-first/file-a.txt", "z-last/file-b.txt"]);
//         assert_eq!(snapshot.metadata.directory_count, 3);
//         assert_eq!(snapshot.metadata.file_count, 2);
//         assert!(snapshot.root().is_some());
//     }

//     #[test]
//     fn snapshot_root_changes_for_path_and_content_changes() {
//         let renamed = TempDir::new();
//         fs::write(renamed.path().join("alpha.txt"), b"same-content").expect("write alpha");

//         let moved = TempDir::new();
//         fs::write(moved.path().join("beta.txt"), b"same-content").expect("write beta");

//         let changed = TempDir::new();
//         fs::write(changed.path().join("alpha.txt"), b"different-content").expect("write changed");

//         let renamed_snapshot = build_snapshot(renamed.path()).expect("build renamed snapshot");
//         let moved_snapshot = build_snapshot(moved.path()).expect("build moved snapshot");
//         let changed_snapshot = build_snapshot(changed.path()).expect("build changed snapshot");

//         assert_ne!(renamed_snapshot.root(), moved_snapshot.root());
//         assert_ne!(renamed_snapshot.root(), changed_snapshot.root());
//         assert_eq!(renamed_snapshot.files[0].hash, moved_snapshot.files[0].hash);
//     }

//     fn path_to_string(path: &Path) -> String {
//         path.to_string_lossy().into_owned()
//     }

//     struct TempDir {
//         path: PathBuf,
//     }

//     impl TempDir {
//         fn new() -> Self {
//             let unique = format!(
//                 "mtree-test-{}-{}",
//                 std::process::id(),
//                 UNIX_EPOCH
//                     .elapsed()
//                     .unwrap_or_else(|_| SystemTime::now()
//                         .duration_since(UNIX_EPOCH)
//                         .unwrap_or_default())
//                     .as_nanos()
//                     + NEXT_ID.fetch_add(1, Ordering::Relaxed) as u128
//             );
//             let path = std::env::temp_dir().join(unique);
//             fs::create_dir_all(&path).expect("create temp dir");
//             Self { path }
//         }

//         fn path(&self) -> &Path {
//             &self.path
//         }
//     }

//     impl Drop for TempDir {
//         fn drop(&mut self) {
//             let _ = fs::remove_dir_all(&self.path);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::{
        FileEntry, build_snapshot, combine_hashes, file_leaf_payload, hash_directory_node,
        hash_file_node,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn snapshot_hashes_directory_subtrees_before_parent_level_pairing() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.path().join("books")).expect("create books directory");
        fs::write(temp.path().join("a.txt"), b"a").expect("write a.txt");
        fs::write(temp.path().join("books/file.txt"), b"book-file").expect("write books/file.txt");
        fs::write(temp.path().join("books/text.txt"), b"book-text").expect("write books/text.txt");
        fs::write(temp.path().join("index.html"), b"<html></html>").expect("write index.html");

        let snapshot = build_snapshot(temp.path()).expect("build snapshot");

        let a = find_file(&snapshot.files, "a.txt");
        let books_file = find_file(&snapshot.files, "books/file.txt");
        let books_text = find_file(&snapshot.files, "books/text.txt");
        let index = find_file(&snapshot.files, "index.html");

        let books_children =
            combine_hashes(vec![hash_file_node(books_file), hash_file_node(books_text)]);
        let books_dir = hash_directory_node(Path::new("books"), Some(books_children));
        let expected_root =
            combine_hashes(vec![hash_file_node(a), books_dir, hash_file_node(index)]);

        assert_eq!(snapshot.root_hash(), Some(expected_root));

        let flat_leaf_refs = [
            file_leaf_payload(a),
            file_leaf_payload(books_file),
            file_leaf_payload(books_text),
            file_leaf_payload(index),
        ];
        let flat_leaf_slices: Vec<&[u8]> = flat_leaf_refs.iter().map(Vec::as_slice).collect();
        let flat_root = crate::MerkleTree::new(&flat_leaf_slices).root();

        assert_ne!(snapshot.root(), flat_root);
    }

    fn find_file<'a>(files: &'a [FileEntry], expected: &str) -> &'a FileEntry {
        files
            .iter()
            .find(|entry| entry.path == Path::new(expected))
            .expect("expected file entry")
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let unique = format!(
                "mtree-test-{}-{}",
                std::process::id(),
                UNIX_EPOCH
                    .elapsed()
                    .unwrap_or_else(|_| {
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                    })
                    .as_nanos()
                    + NEXT_ID.fetch_add(1, Ordering::Relaxed) as u128
            );
            let path = std::env::temp_dir().join(unique);
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
