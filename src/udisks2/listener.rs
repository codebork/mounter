use dbus::blocking::Connection;
use std::time::Duration;
use std::rc::Rc;
use super::dbus_interface;
use crate::udisks2::devices::{Drive, Block};

pub struct Listener {
    drive_added: Rc<Option<Box<dyn Fn(Drive)>>>,
    block_device_added: Rc<Option<Box<dyn Fn(Block)>>>,
    object_removed: Rc<Option<Box<dyn Fn(String)>>>
}

impl Listener {
    pub fn new() -> Self {
        Listener {
            drive_added: Rc::new(None),
            block_device_added: Rc::new(None),
            object_removed: Rc::new(None)
        }
    }

    pub fn drive_added<F: 'static>(&mut self, callback: F)
        where F: Fn(Drive)
    {
        self.drive_added = Rc::new(Some(Box::new(callback)));
    }

    pub fn block_device_added<F: 'static>(&mut self, callback: F)
        where F: Fn(Block)
    {
        self.block_device_added = Rc::new(Some(Box::new(callback)));
    }

    pub fn object_removed<F: 'static>(&mut self, callback: F)
        where F: Fn(String)
    {
        self.object_removed = Rc::new(Some(Box::new(callback)));
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = Connection::new_system().expect("Could not connect to system bus");
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));
        let drive_added = Rc::clone(&self.drive_added);
        let block_device_added = Rc::clone(&self.block_device_added);
        let object_removed = Rc::clone(&self.object_removed);

        proxy.match_signal(move |signal: dbus_interface::OrgFreedesktopDBusObjectManagerInterfacesAdded, _conn: &Connection| {
            if let Some(drive) = Drive::new(&signal.object_path, &signal.interfaces_and_properties) {
                if let Some(new_drive_handler) = &*drive_added {
                    new_drive_handler(drive);
                }
            }

            if let Some(block_device) = Block::new(&signal.object_path, &signal.interfaces_and_properties) {
                if let Some(new_device_handler) = &*block_device_added {
                    new_device_handler(block_device);
                }
            }

            true
        }).expect("Could not listen for Interfaces Added signal");

        proxy.match_signal(move |signal: dbus_interface::OrgFreedesktopDBusObjectManagerInterfacesRemoved, _conn: &Connection| {
            if let Some(removed_object_handler) = &*object_removed {
                removed_object_handler(signal.object_path.to_string());
            }

            true
        }).expect("Could not listen for Interfaces Removed signal");

        loop { conn.process(Duration::from_millis(1000))?; }
    }
}
