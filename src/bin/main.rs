use std::process;
use mounter;

fn main() {
    if let Err(e) = mounter::run() {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

