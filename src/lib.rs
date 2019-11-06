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
        manager_clone.borrow().new_fs(filesystem);
    });

    udisks2_wrapper.run()
}

pub struct Manager;

impl Manager {
    pub fn new() -> Manager {
        return Manager;
    }

    pub fn new_fs(&self, filesystem: Filesystem) {
        Notification::mounted(format!("{}: {}", filesystem.device, filesystem.label.unwrap())).send();
    }

    /*
        let fs = String::from("org.freedesktop.UDisks2.Filesystem");
        if interfaces.contains(&&fs) {
            let fs_object = conn.with_proxy("org.freedesktop.UDisks2", &signal.object_path, Duration::from_millis(5000));
            let options: HashMap<String, Variant<&str>> = HashMap::new();
            println!("{:#?}", signal.interfaces_and_properties);

            let (mount_path,): (String,) = fs_object.method_call("org.freedesktop.UDisks2.Filesystem", "Mount", (options,)).unwrap();

            Notification::mounted(mount_path).send();
        }
    }*/
}
