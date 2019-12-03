use std::process;
use mounter;
use mounter::Config;
use std::env;
use std::path::Path;
extern crate xdg;

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("mounter").unwrap();
    let mut args = env::args();

    args.next();

    let config = match args.next() {
        Some(config_file) => Config::parse(Path::new(&config_file)),
        None => match xdg_dirs.find_config_file("config.toml") {
            Some(config_file) => Config::parse(&config_file),
            None => Config::new()
        }
    };

    if let Err(e) = mounter::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

