use std::{path::Path, println};

use mtree::build_snapshot;

fn main() {
    match build_snapshot(Path::new("../tmp/")) {
        Ok(snapshot) => {
            println!("{:?}", snapshot.tree)
        }
        Err(e) => println!("Error building snapshot: {}", e),
    }
}
