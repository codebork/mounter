use std::process;
use std::env;
use std::path::Path;
extern crate xdg;
use udman;

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("udman").unwrap();
    let mut args = env::args();

    args.next();

    let config = match args.next() {
        Some(config_file) => udman::Config::parse(Path::new(&config_file)),
        None => match xdg_dirs.find_config_file("config.toml") {
            Some(config_file) => udman::Config::parse(&config_file),
            None => udman::Config::new()
        }
    };

    println!("{:?}", config);

    if let Err(e) = udman::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

