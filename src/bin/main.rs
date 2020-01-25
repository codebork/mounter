use std::process;
use std::env;
use std::path::Path;
extern crate xdg;
use mounter;

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("mounter").unwrap();
    let mut args = env::args();

    args.next();

    let config = match args.next() {
        Some(config_file) => mounter::Config::parse(Path::new(&config_file)),
        None => match xdg_dirs.find_config_file("config.toml") {
            Some(config_file) => mounter::Config::parse(&config_file),
            None => mounter::Config::new()
        }
    };

    if let Err(e) = mounter::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

