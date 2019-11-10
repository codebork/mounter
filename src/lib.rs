mod notification;
use notification::Notification;
mod udisks2;
use udisks2::{Udisks2, Filesystem};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Rc::new(RefCell::new(Manager::new()));
    let udisks2_wrapper = Udisks2::new();

    let manager_clone = manager.clone();
    udisks2_wrapper.filesystem_added(move |filesystem: Filesystem| {
        let manager = manager_clone.borrow();
        manager.new_fs(filesystem);
    });

    let manager_clone = manager.clone();
    udisks2_wrapper.filesystem_removed(move |object_path: String| {
        let manager = manager_clone.borrow();
        manager.removed_fs(object_path);
    });

    udisks2_wrapper.run()
}

pub struct Manager {
    filesystems: RefCell<HashMap<String, Filesystem>>
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            filesystems: RefCell::new(HashMap::new())
        }
    }

    pub fn new_fs(&self, filesystem: Filesystem) {
        Notification::new_filesystem(filesystem.details()).send();

        if let Ok(mount_path) = &filesystem.mount() {
            Notification::mounted(&mount_path).send();
        } else {
            Notification::mount_failed(&filesystem.device).send();
        }

        self.filesystems.borrow_mut().insert(filesystem.object_path.to_string(), filesystem);
    }

    pub fn removed_fs(&self, object_path: String) {
        if let Some(filesystem) = self.filesystems.borrow_mut().remove(&object_path) {
            Notification::unmounted(filesystem.device).send();
        }
    }
}
