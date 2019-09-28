use dbus::blocking::Connection;
use dbus::arg::Variant;
use std::time::Duration;
use std::collections::HashMap;
mod udisks2;
mod notification;
use notification::Notification;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = Connection::new_system()?;

    let proxy = conn.with_proxy("org.freedesktop.UDisks2", "/org/freedesktop/UDisks2", Duration::from_millis(5000));

    proxy.match_signal_local(|h: udisks2::OrgFreedesktopDBusObjectManagerInterfacesAdded, conn: &Connection| {
        let interfaces: Vec<&String> = h.interfaces_and_properties.keys().collect();
        let fs = String::from("org.freedesktop.UDisks2.Filesystem");

        if interfaces.contains(&&fs) {
            let fs_object = conn.with_proxy("org.freedesktop.UDisks2", &h.object_path, Duration::from_millis(5000));
            let options: HashMap<String, Variant<&str>> = HashMap::new();

            let (mount_path,): (String,) = fs_object.method_call("org.freedesktop.UDisks2.Filesystem", "Mount", (options,)).unwrap();

            Notification::mounted(mount_path).send();
        }

        true
    }).expect("Could not listen for Interfaces Added signal");

    proxy.match_signal_local(|h: udisks2::OrgFreedesktopDBusObjectManagerInterfacesRemoved, _: &Connection| {
        Notification::unmounted(h.object_path.to_string()).send();

        true
    }).expect("Could not listen for Interfaces Removed signal");

    loop { conn.process(Duration::from_millis(1000))?; }
}
