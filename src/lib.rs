mod notification;
use notification::Notification;
mod udisks2;
use udisks2::{Udisks2, Filesystem};
use std::cell::RefCell;
use std::rc::Rc;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Rc::new(RefCell::new(Manager::new()));
    let udisks2_wrapper = Udisks2::new();
    let manager_clone = manager.clone();

    udisks2_wrapper.new_filesystem(move |filesystem: Filesystem| {
        let manager = manager_clone.borrow();
        manager.new_fs(&filesystem);
    });

    udisks2_wrapper.run()
}

pub struct Manager;

impl Manager {
    pub fn new() -> Manager {
        Manager
    }

    pub fn new_fs(&self, filesystem: &Filesystem) {
        if let Some(label) = filesystem.label.as_ref() {
            Notification::new_filesystem(format!("{}: {}", filesystem.device, label)).send();
        } else {
            Notification::new_filesystem(filesystem.device.to_string()).send();
        }
    }
}
