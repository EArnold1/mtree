use mtree::build_snapshot;
use std::path::Path;

use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    match build_snapshot(Path::new("/mtree/tmp")) {
        Ok(snapshot) => {
            snapshot.save_snapshot("snapshot.json").unwrap();
            println!("{:?}", snapshot.tree)
        }
        Err(e) => println!("{}", e),
    }
}
