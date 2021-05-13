use super::udisks2::{Udisks2ManagedObjects, devices::{Block, Drive, Encrypted, Filesystem}};
use std::collections::HashMap;
use super::notifications::{Notifier};
use super::notices::Notice;
use super::config::Config;
use dialog::DialogBox;

/// Keeps track of and controls devices and drives
#[derive(Debug)]
pub struct Manager {
    config: Config,
    drives: HashMap<String, Drive>,
    devices: HashMap<String, Block>
}

impl Manager {
    /// Create a new instance of manager with the specified configuration.
    /// Initial UDisks2 state can be passed in also this allows udman to keep
    /// track of devices that were attached before the program started running.
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

    /// Inserts a new drive object into the list of monitored drives
    pub fn new_drive(&mut self, drive: Drive) {
        self.drives.insert(drive.object_path.to_string(), drive.to_owned());
    }

    /// Inserts a new block device object into the list of monitored drives
    pub fn new_device(&mut self, device: Block) {
        self.devices.insert(device.object_path.to_string(), device.to_owned());

        if let Some(filesystem) = device.as_fs() {
            self.new_filesystem(filesystem);
        }

        if let Some(encrypted) = device.as_enc() {
            self.new_encrypted(encrypted);
        }
    }

    fn new_encrypted(&mut self, encrypted: Encrypted) {
        // Don't unlock if the device is already decrypted
        if let Some(enc_info) = &encrypted.device.enc_info {
            if enc_info.cleartext_device.is_some() {
                return;
            }
        }

        Notifier::notify(Notice::NewEncrypted(&encrypted.device.device)).ok();

        let mut keyfile_path = None;
        let mut password = None;

        if let Some(encrypted_config) = self.config.get_uuid_settings(&encrypted.device.uuid.as_ref().unwrap()) {
            keyfile_path = encrypted_config.keyfile.to_owned();
            password = encrypted_config.password.to_owned();
        }

        encrypted
            .unlock(keyfile_path, password.or(self.password_prompt()))
            .map_or_else(
                |e| {
                    eprintln!("{}", e);
                    Notifier::notify(Notice::DecryptFail(&e.to_string())).ok();
                },
                |path| {
                    Notifier::notify(Notice::DecryptSuccess(&path)).ok();
                }
            );
    }

    fn password_prompt(&self) -> Option<String> {
        dialog::Password::new("Enter password")
            .title("Encrypted Device")
            .show()
            .unwrap_or(None)
    }

    fn new_filesystem(&mut self, filesystem: Filesystem) {
        // Don't do anything if the drive isn't removable
        if let Some(drive) = self.drives.get(filesystem.device.drive.as_ref().unwrap()) {
            if !drive.removable {
                return;
            }
        }

        // Don't alert and mount if it's already mounted
        if let Some(fs_info) = &filesystem.device.fs_info {
            if fs_info.mount_paths.is_some() {
                return;
            }
        }
 
        Notifier::notify(Notice::NewFilesystem(&filesystem.device.device)).ok();

        let mut should_mount = None;
        let mut script = None;

        if let Some(fs_config) = self.config.get_uuid_settings(&filesystem.device.uuid.as_ref().unwrap()) {
            should_mount = fs_config.automount;
            script = fs_config.run.to_owned();
        }

        if should_mount.unwrap_or(self.config.settings.automount) {
            match filesystem.mount() {
                Ok(mount_path) => {
                    Notifier::notify(Notice::MountSuccess(&mount_path)).ok();

                    if let Some(script) = script {
                        if std::path::Path::new(&script).exists() {
                            std::process::Command::new(script).output().expect("failed to execute command");
                        }
                    }
                },
                Err(e) => {
                    eprintln!("{:#?}", e);
                    Notifier::notify(Notice::MountFail(&filesystem.device.device)).ok();
                }
            }
        }
    }

    /// Removes devices from memory. If the removed device was a filesystem
    /// then a notification is sent with information
    pub fn removed_object(&mut self, object_path: String) {
        if let Some(device) = self.devices.remove(&object_path) {
            if let Some(filesystem) = device.as_fs() {
                Notifier::notify(Notice::UnmountSuccess(&filesystem.device.device)).ok();
            }
        }
    }
}
