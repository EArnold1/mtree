use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{DirectorySnapshot, FileEntry, build_snapshot, error::MtreeError, info};

#[derive(Default)]
struct FileChanges {
    modified: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
    added: Vec<PathBuf>,
}

impl FileChanges {
    fn log(&self) {
        for path in &self.modified {
            info!("Modified file: {}", path.display());
        }
        for path in &self.deleted {
            info!("Deleted file: {}", path.display());
        }
        for path in &self.added {
            info!("Added file: {}", path.display());
        }
    }
}

pub fn execute(live_dir: &Path, snapshot_path: &Path) -> Result<(), MtreeError> {
    let snapshot = DirectorySnapshot::deserialize_snapshot(snapshot_path)?;

    let live_snapshot = build_snapshot(live_dir)?;

    if live_snapshot != snapshot {
        info!("Snapshot has changed");
        classify_file_changes(&live_snapshot.files, &snapshot.files).log();
    } else {
        info!("Snapshot is unchanged");
    }

    Ok(())
}

fn classify_file_changes(current: &[FileEntry], expected: &[FileEntry]) -> FileChanges {
    let expected_by_path: HashMap<&Path, &FileEntry> = expected
        .iter()
        .map(|file| (file.path.as_path(), file))
        .collect();
    let current_paths: HashSet<&Path> = current.iter().map(|file| file.path.as_path()).collect();

    let mut changes = FileChanges::default();

    for file in current {
        match expected_by_path.get(file.path.as_path()) {
            Some(expected_file) if expected_file.content_hash != file.content_hash => {
                changes.modified.push(file.path.clone());
            }
            Some(_) => {}
            None => changes.added.push(file.path.clone()),
        }
    }

    for file in expected {
        if !current_paths.contains(file.path.as_path()) {
            changes.deleted.push(file.path.clone());
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{FileEntry, NodeHash};

    use super::classify_file_changes;

    #[test]
    fn classifies_added_deleted_and_modified_files() {
        let expected = vec![
            file("deleted.txt", 1),
            file("modified.txt", 2),
            file("unchanged.txt", 3),
        ];
        let current = vec![
            file("added.txt", 4),
            file("modified.txt", 5),
            file("unchanged.txt", 3),
        ];

        let changes = classify_file_changes(&current, &expected);

        assert_paths(&changes.modified, &["modified.txt"]);
        assert_paths(&changes.deleted, &["deleted.txt"]);
        assert_paths(&changes.added, &["added.txt"]);
    }

    #[test]
    fn treats_a_move_as_an_addition_and_deletion() {
        let expected = vec![file("before.txt", 1)];
        let current = vec![file("after.txt", 1)];

        let changes = classify_file_changes(&current, &expected);

        assert!(changes.modified.is_empty());
        assert_paths(&changes.deleted, &["before.txt"]);
        assert_paths(&changes.added, &["after.txt"]);
    }

    fn file(path: &str, hash_byte: u8) -> FileEntry {
        FileEntry {
            path: PathBuf::from(path),
            content_hash: NodeHash::from_array([hash_byte; 32]),
            size: 0,
        }
    }

    fn assert_paths(paths: &[PathBuf], expected: &[&str]) {
        let actual: Vec<_> = paths.iter().map(|path| path.to_string_lossy()).collect();
        assert_eq!(actual, expected);
    }
}
