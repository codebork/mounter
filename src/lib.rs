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
pub struct FsSettings {
    automount: Option<bool>,
    command: Option<String>
}

#[derive(Debug, Deserialize)]
// Sets all fields to default() values and
// overwrites ones that are present in toml file
#[serde(default)]
pub struct Settings {
    automount: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            automount: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    settings: Settings,
    filesystems: Option<HashMap<String, FsSettings>>
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: Settings::default(),
            filesystems: None
        }
    }

    pub fn parse(path: &Path) -> Self {
        // If the file can be read then parse it otherwise
        // return an empty config file
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();

            // Read config file into string and convert into Config struct
            file.read_to_string(&mut contents).expect("Could not read file");
            toml::from_str(contents.as_str()).unwrap()
        } else {
            eprintln!("Could not read config file: {:?}", path);
            Config::new()
        }
    }
}

// TODO
// Handle encrypted devices
// Handle devices already connected at start up

pub struct Manager {
    config: Config,
    filesystems: HashMap<String, Filesystem>
}

impl Manager {
    pub fn new(config: Config) -> Manager {
        // println!("{:#?}", config);
        Manager {
            config: config,
            filesystems: HashMap::new()
        }
    }

    fn get_fs_settings(&self, uuid: &str) -> Option<&FsSettings> {
        self.config.filesystems.as_ref()?.get(uuid)
    }

    pub fn new_fs(&mut self, filesystem: Filesystem) {
        Notification::new_filesystem(filesystem.details()).send();

        match self.get_fs_settings(&filesystem.uuid) {
            Some(filesystem_config) => {
                let should_mount = filesystem_config.automount.unwrap_or(self.config.settings.automount);

                if should_mount {
                    if let Ok(mount_path) = &filesystem.mount() {
                        Notification::mounted(&mount_path).send();

                        if let Some(command) = &filesystem_config.command {
                            if Path::new(command).exists() {
                                Command::new(command).output().expect("failed to execute command");
                            }
                        }
                    } else {
                        Notification::mount_failed(&filesystem.device).send();
                    }
                }
            },
            None => {
                if self.config.settings.automount {
                    if let Ok(mount_path) = &filesystem.mount() {
                        Notification::mounted(&mount_path).send();
                    } else {
                        Notification::mount_failed(&filesystem.device).send();
                    }
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
