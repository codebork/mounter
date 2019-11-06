mod udisks2_dbus;
use dbus::blocking::Connection;
use std::time::Duration;
use std::cell::RefCell;

pub struct Filesystem {
    pub object_path: String,
    pub device: String,
    pub label: Option<String>
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


    pub fn new_filesystem<F: 'static>(&self, callback: F)
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
                    object_path: signal.object_path.to_string(),
                    device: String::from_utf8(device).unwrap().trim_matches(char::from(0)).to_string(),
                    label: Some(label)
                };
                callback(fs);
            }

            true
        }).expect("Could not listen for Interfaces Added signal");
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop { self.conn.borrow_mut().process(Duration::from_millis(1000))?; }
    }
}
