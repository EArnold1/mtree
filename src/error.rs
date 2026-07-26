use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MtreeError {
    #[error("I/O error while building snapshot: {0}")]
    Io(#[from] io::Error),
    #[error("snapshot root must be an existing directory: {0}")]
    InvalidRoot(PathBuf),
    #[error("unsupported directory entry type: {0}")]
    UnsupportedEntry(PathBuf),
    #[error("failed to derive a relative path for {path} from root {root}")]
    PathPrefix { path: PathBuf, root: PathBuf },
    #[error("Serde JSON error while parsing snapshot: {0}")]
    ParseError(#[from] serde_json::Error),
}
