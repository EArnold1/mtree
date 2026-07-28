use mtree::{commands, error};

fn main() {
    if let Err(err) = commands::run() {
        error!("{err}");
        std::process::exit(1);
    }
}
