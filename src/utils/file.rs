use std::path::{Path, PathBuf};

use crate::error::MtreeError;

pub fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn relative_path(root: &Path, path: &Path) -> Result<PathBuf, MtreeError> {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| MtreeError::PathPrefix {
            path: path.to_path_buf(),
            root: root.to_path_buf(),
        })
}
