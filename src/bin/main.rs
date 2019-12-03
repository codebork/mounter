use std::process;
use mounter;
use mounter::Config;

fn main() {
    let config = Config::parse("config.toml".to_string());

    if let Err(e) = mounter::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

