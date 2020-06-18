use dbus::blocking::Connection;
use dbus::arg::{Variant, RefArg};
use std::collections::HashMap;
mod dbus_interface;
use super::block;
use dbus_interface::UDisks2Filesystem;

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
        let conn = Connection::new_system()?;
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", &self.device.object_path, std::time::Duration::from_millis(5000));
        let options: HashMap<&str, Variant<std::boxed::Box<(dyn RefArg + 'static)>>> = HashMap::new();

        proxy.mount(options)
    }
}
