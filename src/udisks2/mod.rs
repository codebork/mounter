mod udisks2_dbus;
use dbus::blocking::Connection;
use std::time::Duration;
use std::cell::RefCell;
use std::collections::HashMap;
use dbus::arg::{RefArg, Variant};
use dbus::strings::Path;
use crate::udisks2::udisks2_dbus::OrgFreedesktopDBusObjectManager;

pub type Udisks2InterfacesAndProps = HashMap<String, HashMap<String, Variant<std::boxed::Box<(dyn RefArg + 'static)>>>>;
pub type Udisks2ManagedObjects = HashMap<Path<'static>, Udisks2InterfacesAndProps>;

pub struct Filesystem {
    pub uuid: String,
    pub object_path: String,
    pub device: String,
    pub label: Option<String>
}

impl Filesystem {
    pub fn new(object_path: &Path, interfaces_and_properties: &Udisks2InterfacesAndProps) -> Filesystem {
        let label = interfaces_and_properties["org.freedesktop.UDisks2.Block"]["IdLabel"].0.as_str().unwrap().to_string();
        let device: Vec<u8> = interfaces_and_properties["org.freedesktop.UDisks2.Block"]["Device"].0
            .as_iter().unwrap().map(|r| r.as_u64().unwrap() as u8).collect();

        Filesystem {
            uuid: interfaces_and_properties["org.freedesktop.UDisks2.Block"]["IdUUID"].0.as_str().unwrap().to_string(),
            object_path: object_path.to_string(),
            device: String::from_utf8(device).unwrap().trim_matches(char::from(0)).to_string(),
            label: Some(label)
        }
    }

    pub fn details(&self) -> String {
        if let Some(label) = &self.label {
            format!("{}: {}", self.device, label)
        } else {
            self.device.to_string()
        }
    }

    pub fn mount(&self) -> Result<String, Box<dyn std::error::Error>> {
        let conn = Connection::new_system().expect("Could not connect to system bus");
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", &self.object_path, Duration::from_millis(5000));
        let options: HashMap<String, Variant<&str>> = HashMap::new();

        let (mount_path,): (String,) = proxy.method_call("org.freedesktop.UDisks2.Filesystem", "Mount", (options,))?;

        Ok(mount_path)
    }
}

pub struct Udisks2 {
    conn: RefCell<Connection>
}

impl Udisks2 {
    pub fn new() -> Self {
        Udisks2 {
            conn: RefCell::new(Connection::new_system().expect("Could not connect to system bus")),
        }
    }

    pub fn current_state(&self) -> Udisks2ManagedObjects {
        let conn = self.conn.borrow();
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

        proxy.get_managed_objects().unwrap()
    }

    pub fn filesystem_added<F: 'static>(&self, callback: F)
        where F: Fn(Filesystem)
    {
        let conn = self.conn.borrow();
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

        proxy.match_signal_local(move |signal: udisks2_dbus::OrgFreedesktopDBusObjectManagerInterfacesAdded, _conn: &Connection| {
            let fs_interface = String::from("org.freedesktop.UDisks2.Filesystem");
            let interfaces: Vec<&String> = signal.interfaces_and_properties.keys().collect();

            if interfaces.contains(&&fs_interface) {
                let label = signal.interfaces_and_properties["org.freedesktop.UDisks2.Block"]["IdLabel"].0.as_str().unwrap().to_string();
                let device: Vec<u8> = signal.interfaces_and_properties["org.freedesktop.UDisks2.Block"]["Device"].0
                    .as_iter().unwrap().map(|r| r.as_u64().unwrap() as u8).collect();
                let fs = Filesystem {
                    uuid: signal.interfaces_and_properties["org.freedesktop.UDisks2.Block"]["IdUUID"].0.as_str().unwrap().to_string(),
                    object_path: signal.object_path.to_string(),
                    device: String::from_utf8(device).unwrap().trim_matches(char::from(0)).to_string(),
                    label: Some(label)
                };
                callback(fs);
            }

            true
        }).expect("Could not listen for Interfaces Added signal");

    }

    pub fn filesystem_removed<F: 'static>(&self, callback: F)
        where F: Fn(String)
    {
        let conn = self.conn.borrow();
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

        proxy.match_signal_local(move |signal: udisks2_dbus::OrgFreedesktopDBusObjectManagerInterfacesRemoved, _conn: &Connection| {
            let fs_interface = String::from("org.freedesktop.UDisks2.Filesystem");

            if signal.interfaces.contains(&&fs_interface) {
                callback(signal.object_path.to_string());
            }

            true
        }).expect("Could not listen for Interfaces Removed signal");
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop { self.conn.borrow_mut().process(Duration::from_millis(1000))?; }
    }
}
