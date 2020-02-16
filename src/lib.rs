mod notification;
mod udisks2;
use udisks2::{Udisks2ManagedObjects, Block, Drive};
use std::collections::HashMap;
mod config;
pub use config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut udisks2_listener = udisks2::Listener::new();
    let manager = std::rc::Rc::new(std::cell::RefCell::new(Manager::new(config, Some(udisks2::current_state()))));

    let manager_clone = manager.clone();
    udisks2_listener.drive_added(move |drive: Drive| {
        let mut manager = manager_clone.borrow_mut();
        manager.new_drive(drive);
    });

    let manager_clone = manager.clone();
    udisks2_listener.block_device_added(move |block_device: Block| {
        let mut manager = manager_clone.borrow_mut();
        manager.new_device(block_device);
    });

    let manager_clone = manager.clone();
    udisks2_listener.object_removed(move |object_path: String| {
        let mut manager = manager_clone.borrow_mut();
        manager.removed_object(object_path);
    });

    udisks2_listener.run()
}

// TODO
// Password/key file input for encrypted devices
// Add check for if filesystem was mounted before issuing unmounted notification

#[derive(Debug)]
pub struct Manager {
    config: Config,
    drives: HashMap<String, Drive>,
    devices: HashMap<String, Block>
}

impl Manager {
    pub fn new(config: Config, initial_state: Option<Udisks2ManagedObjects>) -> Manager {
        let mut new_manager = Manager {
            config: config,
            drives: HashMap::new(),
            devices: HashMap::new()
        };

        if let Some(initial_state) = initial_state {
            new_manager.parse_initial_udisks2_state(initial_state);
        };

        new_manager
    }

    fn parse_initial_udisks2_state(&mut self, initial_state: Udisks2ManagedObjects) {
        let mut drives: Vec<Drive> = Vec::new();
        let mut devices: Vec<Block> = Vec::new();

        for (object_path, interfaces_and_properties) in initial_state.iter() {
            if let Some(drive) = Drive::new(&object_path, &interfaces_and_properties) {
                drives.push(drive);
            }

            if let Some(block_device) = Block::new(&object_path, &interfaces_and_properties) {
                devices.push(block_device);
            }
        }

        for drive in drives {
            self.new_drive(drive);
        }

        for device in devices {
            self.new_device(device);
        }
    }

    pub fn new_drive(&mut self, drive: Drive) {
        self.drives.insert(drive.object_path.to_string(), drive.to_owned());
    }

    pub fn new_device(&mut self, device: Block) {
        self.devices.insert(device.object_path.to_string(), device.to_owned());

        if let Some(filesystem) = device.as_fs() {
            self.new_filesystem(filesystem);
        }

        if let Some(encrypted) = device.as_enc() {
            self.new_encrypted(encrypted);
        }
    }

    fn new_encrypted(&mut self, encrypted: udisks2::Encrypted) {
        match encrypted.unlock("".to_string()) {
            Ok(path) => println!("Cleartext: {}", path),
            Err(e) => eprintln!("{:#?}", e)
        }
    }

    fn new_filesystem(&mut self, filesystem: udisks2::Filesystem) {
        if let Some(drive) = self.drives.get(filesystem.device.drive.as_ref().unwrap()) {
            if !drive.removable {
                return
            }
        } else {
            println!("No drive");
            return
        }

        notification::new_filesystem(&filesystem.device.device).send();

        match self.config.get_fs_settings(&filesystem.device.uuid.as_ref().unwrap()) {
            Some(filesystem_config) => {
                let should_mount = filesystem_config.automount.unwrap_or(self.config.settings.automount);

                if should_mount {
                    match filesystem.mount() {
                        Ok(mount_path) => {
                            notification::mounted(&mount_path).send();

                            if let Some(command) = &filesystem_config.command {
                                if std::path::Path::new(command).exists() {
                                    std::process::Command::new(command).output().expect("failed to execute command");
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("{:#?}", e);
                            notification::mount_failed(&filesystem.device.device).send();
                        }
                    }
                }
            },
            None => {
                if self.config.settings.automount {
                    if let Ok(mount_path) = filesystem.mount() {
                        notification::mounted(&mount_path).send();
                    } else {
                        notification::mount_failed(&filesystem.device.uuid.unwrap_or_default()).send();
                    }
                }
            }
        }

    }

    pub fn removed_object(&mut self, object_path: String) {
        if let Some(device) = self.devices.remove(&object_path) {
            if let Some(filesystem) = device.as_fs() {
                notification::unmounted(&filesystem.device.device).send();
            }
        }
    }
}
