use env_logger::Env;
use mtree::commands;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    if let Err(err) = commands::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
