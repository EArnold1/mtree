use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::{Path, PathBuf},
};

use crate::{DirectorySnapshot, FileEntry, error::MtreeError};

#[derive(Default)]
pub(crate) struct FileChanges {
    modified: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
    added: Vec<PathBuf>,
}

impl fmt::Display for FileChanges {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Changes detected")?;

        write_change_section(formatter, "Modified", 'M', &self.modified)?;
        write_change_section(formatter, "Added", 'A', &self.added)?;
        write_change_section(formatter, "Deleted", 'D', &self.deleted)?;

        writeln!(
            formatter,
            "\nSummary: {} modified, {} added, {} deleted",
            self.modified.len(),
            self.added.len(),
            self.deleted.len(),
        )
    }
}

fn write_change_section(
    formatter: &mut fmt::Formatter<'_>,
    title: &str,
    status: char,
    paths: &[PathBuf],
) -> fmt::Result {
    if paths.is_empty() {
        return Ok(());
    }

    writeln!(formatter, "\n{title} ({})", paths.len())?;
    for path in paths {
        writeln!(formatter, "  {status}  {}", path.display())?;
    }

    Ok(())
}

pub fn execute(
    baseline_snapshot_path: &Path,
    comparison_snapshot_path: &Path,
) -> Result<(), MtreeError> {
    let baseline_snapshot = DirectorySnapshot::deserialize_snapshot(baseline_snapshot_path)?;
    let comparison_snapshot = DirectorySnapshot::deserialize_snapshot(comparison_snapshot_path)?;

    let changes = snapshot_diff(&baseline_snapshot, &comparison_snapshot);
    print!("{changes}");
    Ok(())
}

/// Compares a snapshot against a baseline snapshot.
///
/// # Arguments
///
/// * `baseline_snapshot` - The expected state. Files absent from this snapshot but present in
///   `comparison_snapshot` are reported as added; files present here but absent from the
///   comparison are reported as deleted.
/// * `comparison_snapshot` - The state being evaluated against `baseline_snapshot`. A file at
///   the same path with a different content hash is reported as modified.
///
/// Returns an empty [`FileChanges`] when the snapshots have identical Merkle roots.
pub(crate) fn snapshot_diff(
    baseline_snapshot: &DirectorySnapshot,
    comparison_snapshot: &DirectorySnapshot,
) -> FileChanges {
    if baseline_snapshot == comparison_snapshot {
        FileChanges::default()
    } else {
        classify_file_changes(&baseline_snapshot.files, &comparison_snapshot.files)
    }
}

/// Classifies file-level changes from a baseline to a comparison state.
///
/// # Arguments
///
/// * `baseline_files` - Files from the expected state. A path present here but absent from
///   `comparison_files` is deleted.
/// * `comparison_files` - Files from the state being evaluated. A path absent from
///   `baseline_files` is added; a shared path with a different content hash is modified.
///
/// # Returns
///
/// The added, deleted, and modified paths needed to transform `baseline_files` into
/// `comparison_files`.
fn classify_file_changes(
    baseline_files: &[FileEntry],
    comparison_files: &[FileEntry],
) -> FileChanges {
    let baseline_by_path: HashMap<&Path, &FileEntry> = baseline_files
        .iter()
        .map(|file| (file.path.as_path(), file))
        .collect();
    let comparison_paths: HashSet<&Path> = comparison_files
        .iter()
        .map(|file| file.path.as_path())
        .collect();

    let mut changes = FileChanges::default();

    for file in comparison_files {
        match baseline_by_path.get(file.path.as_path()) {
            Some(baseline_file) if baseline_file.content_hash != file.content_hash => {
                changes.modified.push(file.path.clone());
            }
            Some(_) => {}
            None => changes.added.push(file.path.clone()),
        }
    }

    for file in baseline_files {
        if !comparison_paths.contains(file.path.as_path()) {
            changes.deleted.push(file.path.clone());
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{DirectorySnapshot, FileEntry, NodeHash, SnapshotMetadata};

    use super::{classify_file_changes, snapshot_diff};

    #[test]
    fn classifies_added_deleted_and_modified_files() {
        let baseline = vec![
            file("deleted.txt", 1),
            file("modified.txt", 2),
            file("unchanged.txt", 3),
        ];
        let comparison = vec![
            file("added.txt", 4),
            file("modified.txt", 5),
            file("unchanged.txt", 3),
        ];

        let changes = classify_file_changes(&baseline, &comparison);

        assert_paths(&changes.modified, &["modified.txt"]);
        assert_paths(&changes.deleted, &["deleted.txt"]);
        assert_paths(&changes.added, &["added.txt"]);
    }

    #[test]
    fn treats_a_move_as_an_addition_and_deletion() {
        let baseline = vec![file("before.txt", 1)];
        let comparison = vec![file("after.txt", 1)];

        let changes = classify_file_changes(&baseline, &comparison);

        assert!(changes.modified.is_empty());
        assert_paths(&changes.deleted, &["before.txt"]);
        assert_paths(&changes.added, &["after.txt"]);
    }

    #[test]
    fn reports_changes_from_the_baseline_to_the_comparison_snapshot() {
        let baseline = snapshot(vec![file("removed.txt", 1)], 1);
        let comparison = snapshot(vec![file("added.txt", 2)], 2);

        let changes = snapshot_diff(&baseline, &comparison);

        assert!(changes.modified.is_empty());
        assert_paths(&changes.deleted, &["removed.txt"]);
        assert_paths(&changes.added, &["added.txt"]);
    }

    fn file(path: &str, hash_byte: u8) -> FileEntry {
        FileEntry {
            path: PathBuf::from(path),
            content_hash: NodeHash::from_array([hash_byte; 32]),
            size: 0,
        }
    }

    fn snapshot(files: Vec<FileEntry>, tree_hash_byte: u8) -> DirectorySnapshot {
        DirectorySnapshot {
            metadata: SnapshotMetadata {
                root: PathBuf::from("/snapshot-root"),
                generated_at_unix_seconds: 0,
                file_count: files.len(),
            },
            files,
            tree: Some(NodeHash::from_array([tree_hash_byte; 32])),
        }
    }

    fn assert_paths(paths: &[PathBuf], expected: &[&str]) {
        let actual: Vec<_> = paths.iter().map(|path| path.to_string_lossy()).collect();
        assert_eq!(actual, expected);
    }
}
