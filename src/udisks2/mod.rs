mod udisks2_dbus;
use dbus::blocking::Connection;
use std::time::Duration;
use std::collections::HashMap;
use dbus::arg::{RefArg, Variant};
use dbus::strings::Path;
use udisks2_dbus::OrgFreedesktopDBusObjectManager;
mod block;
mod drive;
mod listener;
mod encrypted;
mod filesystem_dbus;
use crate::udisks2::filesystem_dbus::UDisks2Filesystem;

pub use listener::Listener;
pub use block::Block;
pub use drive::Drive;
pub use encrypted::Encrypted;

pub type Udisks2InterfacesAndProps = HashMap<String, HashMap<String, Variant<std::boxed::Box<(dyn RefArg + 'static)>>>>;
pub type Udisks2ManagedObjects = HashMap<Path<'static>, Udisks2InterfacesAndProps>;

pub fn current_state() -> Udisks2ManagedObjects {
    let conn = Connection::new_system().expect("Could not connect to system bus");
    let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

    proxy.get_managed_objects().unwrap()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Interface {
    Filesystem,
    Encrypted
}

pub struct Filesystem {
    pub device: block::Block,
}

impl Filesystem {
    pub fn mount(&self) -> Result<String, dbus::Error> {
        if let Some(info) = &self.device.fs_info {
            if let Some(mount_paths) = &info.mount_paths {
                return Ok(mount_paths[0].to_owned())
            }
        }
        let conn = Connection::new_system().expect("Could not connect to system bus");
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", &self.device.object_path, std::time::Duration::from_millis(5000));
        let options: HashMap<&str, Variant<std::boxed::Box<(dyn RefArg + 'static)>>> = HashMap::new();

        proxy.mount(options)
    }
}

