use dbus::blocking::Connection;
use std::time::Duration;
mod notification;

use dbus::arg::Variant;
use std::collections::HashMap;
use std::str;
use crate::notification::Notification;
mod udisks2;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Manager::new();

    manager.run();

    Ok(())
}

pub struct Manager;

impl Manager {
    pub fn new() -> Manager {
        return Manager;
    }

    pub fn interface_added(
        signal: &udisks2::OrgFreedesktopDBusObjectManagerInterfacesAdded,
        conn: &Connection
    ) {
        let interfaces: Vec<&String> = signal.interfaces_and_properties.keys().collect();
        let fs = String::from("org.freedesktop.UDisks2.Filesystem");
        
        /*
        println!("{:#?}", interfaces);
        println!("{}\n", signal.object_path);

        if interfaces.contains(&&String::from("org.freedesktop.UDisks2.Block")) {
            let value: Vec<u8> = signal.interfaces_and_properties["org.freedesktop.UDisks2.Block"]["Device"].0
                .as_iter().unwrap().map(|r| r.as_u64().unwrap() as u8).collect();
            println!("{}\n", String::from_utf8(value).unwrap().trim_matches(char::from(0)));
        }
        */

        if interfaces.contains(&&fs) {
            let fs_object = conn.with_proxy("org.freedesktop.UDisks2", &signal.object_path, Duration::from_millis(5000));
            let options: HashMap<String, Variant<&str>> = HashMap::new();

            let (mount_path,): (String,) = fs_object.method_call("org.freedesktop.UDisks2.Filesystem", "Mount", (options,)).unwrap();

            Notification::mounted(mount_path).send();
        }
    }

    pub fn interface_removed(
        signal: &udisks2::OrgFreedesktopDBusObjectManagerInterfacesRemoved,
        _conn: &Connection
    ) {
        let fs = String::from("org.freedesktop.UDisks2.Filesystem");

        if signal.interfaces.contains(&&fs) {
            Notification::unmounted(signal.object_path.to_string()).send();
        }

    }

    pub fn run(&self) {
        let mut conn = Connection::new_system().expect("Could not connect to system bus");
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));
        proxy.match_signal_local(|h: udisks2::OrgFreedesktopDBusObjectManagerInterfacesAdded, conn: &Connection| {
            Self::interface_added(&h, conn);

            true
        }).expect("Could not listen for Interfaces Added signal");

        proxy.match_signal_local(|h: udisks2::OrgFreedesktopDBusObjectManagerInterfacesRemoved, conn: &Connection| {
            Self::interface_removed(&h, conn);

            true
        }).expect("Could not listen for Interfaces Removed signal");

        loop { conn.process(Duration::from_millis(1000)).expect("Couldn't process"); }
    }
}
