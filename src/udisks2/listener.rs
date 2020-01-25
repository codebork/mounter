use dbus::blocking::Connection;
use std::time::Duration;
use std::cell::RefCell;
use crate::block_devices;
use crate::udisks2::udisks2_dbus;

pub struct Listener {
    conn: RefCell<Connection>
}

impl Listener {
    pub fn new() -> Self {
        Listener {
            conn: RefCell::new(Connection::new_system().expect("Could not connect to system bus")),
        }
    }

    pub fn block_device_added<F: 'static>(&self, callback: F)
        where F: Fn(block_devices::Block)
    {
        let conn = self.conn.borrow();
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

        proxy.match_signal_local(move |signal: udisks2_dbus::OrgFreedesktopDBusObjectManagerInterfacesAdded, _conn: &Connection| {
            if let Some(block_device) = block_devices::get(&signal.object_path, &signal.interfaces_and_properties) {
                callback(block_device);
            }

            true
        }).expect("Could not listen for Interfaces Added signal");
    }

    pub fn block_device_removed<F: 'static>(&self, callback: F)
        where F: Fn(String)
    {
        let conn = self.conn.borrow();
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

        proxy.match_signal_local(move |signal: udisks2_dbus::OrgFreedesktopDBusObjectManagerInterfacesRemoved, _conn: &Connection| {
            callback(signal.object_path.to_string());

            true
        }).expect("Could not listen for Interfaces Removed signal");
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop { self.conn.borrow_mut().process(Duration::from_millis(1000))?; }
    }
}
