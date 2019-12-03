mod notification;
use notification::Notification;
mod udisks2;
use udisks2::{Udisks2, Filesystem};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::fs::File;
use std::io::prelude::*;
use serde::Deserialize;

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let manager = Rc::new(RefCell::new(Manager::new(config)));
    let udisks2_wrapper = Udisks2::new();

    let manager_clone = manager.clone();
    udisks2_wrapper.filesystem_added(move |filesystem: Filesystem| {
        let mut manager = manager_clone.borrow_mut();
        manager.new_fs(filesystem);
    });

    let manager_clone = manager.clone();
    udisks2_wrapper.filesystem_removed(move |object_path: String| {
        let mut manager = manager_clone.borrow_mut();
        manager.removed_fs(object_path);
    });

    udisks2_wrapper.run()
}

#[derive(Debug, Deserialize)]
pub struct FsOptions {
    command: Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Config {
    filesystems: HashMap<String, FsOptions>
}

impl Config {
    pub fn new() -> Self {
        Config {
            filesystems: HashMap::new()
        }
    }

    pub fn parse(path: String) -> Self {
        // If the file can be read then parse it otherwise
        // return an empty config file
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();

            // Read config file into string and convert into Config struct
            file.read_to_string(&mut contents).expect("Could not read file");
            toml::from_str(contents.as_str()).unwrap()
        } else {
            eprintln!("Could not read config file");
            Config::new()
        }
    }
}

pub struct Manager {
    config: Config,
    filesystems: HashMap<String, Filesystem>
}

impl Manager {
    pub fn new(config: Config) -> Manager {
        Manager {
            config: config,
            filesystems: HashMap::new()
        }
    }

    pub fn new_fs(&mut self, filesystem: Filesystem) {
        Notification::new_filesystem(filesystem.details()).send();

        if let Ok(mount_path) = &filesystem.mount() {
            Notification::mounted(&mount_path).send();
        } else {
            Notification::mount_failed(&filesystem.device).send();
        }

        if let Some(filesystem_config) = self.config.filesystems.get(&filesystem.uuid) {
            if let Some(command) = &filesystem_config.command {
                if Path::new(command).exists() {
                    Command::new(command).output().expect("failed to execute command");
                }
            }
        }

        self.filesystems.insert(filesystem.object_path.to_string(), filesystem);
    }

    pub fn removed_fs(&mut self, object_path: String) {
        if let Some(filesystem) = self.filesystems.remove(&object_path) {
            Notification::unmounted(filesystem.device).send();
        }
    }
}
