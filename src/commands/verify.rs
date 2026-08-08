use std::path::Path;

use crate::{DirectorySnapshot, build_snapshot, commands::diff, error::MtreeError};

pub fn execute(live_dir: &Path, snapshot_path: &Path) -> Result<(), MtreeError> {
    let live_snapshot = build_snapshot(live_dir)?;

    let baseline_snapshot = DirectorySnapshot::deserialize_snapshot(snapshot_path)?;

    let changes = diff::snapshot_diff(&baseline_snapshot, &live_snapshot);
    print!("{changes}");
    Ok(())
}
