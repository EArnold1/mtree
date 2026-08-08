use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::{Path, PathBuf},
    writeln,
};

use crate::{DirectorySnapshot, FileEntry, build_snapshot, error::MtreeError};

#[derive(Default)]
struct FileChanges {
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

pub fn execute(live_dir: &Path, snapshot_path: &Path) -> Result<(), MtreeError> {
    let snapshot = DirectorySnapshot::deserialize_snapshot(snapshot_path)?;

    let live_snapshot = build_snapshot(live_dir)?;

    if live_snapshot != snapshot {
        let changes = classify_file_changes(&live_snapshot.files, &snapshot.files);
        print!("{changes}");
    } else {
        println!("No changes detected");
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

    #[test]
    fn formats_changes_as_a_grouped_cli_report() {
        let changes = classify_file_changes(
            &[file("added.txt", 1), file("modified.txt", 2)],
            &[file("deleted.txt", 3), file("modified.txt", 4)],
        );

        assert_eq!(
            changes.to_string(),
            concat!(
                "verify: changes detected\n\n",
                "Modified (1)\n",
                "  M  modified.txt\n\n",
                "Added (1)\n",
                "  A  added.txt\n\n",
                "Deleted (1)\n",
                "  D  deleted.txt\n\n",
                "Summary: 1 modified, 1 added, 1 deleted\n",
            ),
        );
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
