mod notification;
mod udisks2;
use udisks2::{Udisks2ManagedObjects, block_devices};
use std::collections::HashMap;
mod config;
pub use config::Config;


pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let udisks2_listener = udisks2::Listener::new();
    let manager = std::rc::Rc::new(std::cell::RefCell::new(Manager::new(config, Some(udisks2::current_state()))));

    let manager_clone = manager.clone();
    udisks2_listener.block_device_added(move |block_device: block_devices::Block| {
        let mut manager = manager_clone.borrow_mut();
        manager.new_device(block_device);
    });

    let manager_clone = manager.clone();
    udisks2_listener.block_device_removed(move |object_path: String| {
        let mut manager = manager_clone.borrow_mut();
        manager.removed_device(object_path);
    });

    udisks2_listener.run()
}


// TODO
// Password input for encrypted devices
// Run on removable drives only

pub struct Manager {
    config: Config,
    devices: HashMap<String, block_devices::Block>
}

impl Manager {
    pub fn new(config: Config, initial_state: Option<Udisks2ManagedObjects>) -> Manager {
        let mut new_manager = Manager {
            config: config,
            devices: HashMap::new()
        };

        if let Some(initial_state) = initial_state {
            new_manager.parse_initial_udisks2_state(initial_state);
        };

        new_manager
    }

    fn parse_initial_udisks2_state(&mut self, initial_state: Udisks2ManagedObjects) {
        for (object_path, interfaces_and_properties) in initial_state.iter() {
            if let Some(block_device) = block_devices::get(&object_path, &interfaces_and_properties) {
                self.new_device(block_device);
            }
        }
    }

    fn store_block_device(&mut self, device: &block_devices::Block) {
        self.devices.insert(device.object_path.to_string(), device.to_owned());
    }

    pub fn new_device(&mut self, device: block_devices::Block) {
        self.store_block_device(&device);

        if device.has_interface(block_devices::Interface::Filesystem) {
            self.new_filesystem(device.as_fs());
        }

        if device.has_interface(block_devices::Interface::Encrypted) {
            self.new_encrypted(device.as_enc());
        }
    }

    fn new_encrypted(&mut self, encrypted: udisks2::Encrypted) {
        match encrypted.unlock("".to_string()) {
            Ok(path) => println!("Cleartext: {}", path),
            Err(e) => eprintln!("{:#?}", e)
        }
    }

    fn new_filesystem(&mut self, filesystem: udisks2::Filesystem) {
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

    pub fn removed_device(&mut self, object_path: String) {
        if let Some(filesystem) = self.devices.remove(&object_path) {
            notification::unmounted(&filesystem.device).send();
        }
    }
}
