mod dbus_interface;
use dbus::blocking::Connection;
use std::time::Duration;
use std::collections::HashMap;
use dbus::arg::{RefArg, Variant};
use dbus::strings::Path;
use dbus_interface::OrgFreedesktopDBusObjectManager;
pub mod devices;
mod listener;
pub use listener::Listener;

pub type Udisks2InterfacesAndProps = HashMap<String, HashMap<String, Variant<std::boxed::Box<(dyn RefArg + 'static)>>>>;
pub type Udisks2ManagedObjects = HashMap<Path<'static>, Udisks2InterfacesAndProps>;

pub fn current_state() -> Result<Udisks2ManagedObjects, dbus::Error> {
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

    proxy.get_managed_objects()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Interface {
    Filesystem,
    Encrypted
}
