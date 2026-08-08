use std::path::Path;

use crate::{DirectorySnapshot, build_snapshot, error::MtreeError, info};

pub fn execute(dir: &Path, snapshot_path: &Path) -> Result<(), MtreeError> {
    let dir_snapshot = build_snapshot(dir)?;

    let snapshot = DirectorySnapshot::deserialize_snapshot(snapshot_path)?;

    let is_changed = dir_snapshot != snapshot;

    if is_changed {
        info!("Snapshot has changed");
    } else {
        info!("Snapshot is unchanged");
    }

    Ok(())
}
