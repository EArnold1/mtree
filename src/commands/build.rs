use std::path::Path;

use crate::{build_snapshot, error::MtreeError};

pub fn execute(dir: &Path, output: Option<&Path>) -> Result<(), MtreeError> {
    let snapshot = build_snapshot(dir)?;

    if let Some(path) = output {
        return snapshot.save_snapshot(path);
    }

    let json = serde_json::to_string(&snapshot)?;

    // println!() over info!() because info!() adds additional metadata to the output.
    println!("{json}");
    Ok(())
}
