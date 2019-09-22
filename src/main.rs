use dbus::blocking::Connection;
use dbus::arg::{Variant, Array, Dict};
use std::time::Duration;
use std::collections::HashMap;
mod udisks2;

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

            notify_mounted(&mount_path).unwrap();
        }

        true
    }).expect("Could not listen for Interfaces Added signal");

    proxy.match_signal_local(|h: udisks2::OrgFreedesktopDBusObjectManagerInterfacesRemoved, _: &Connection| {
        // println!("Device unmounted");
        // println!("{:#?}", h.interfaces);
        // println!("{}", h.object_path);

        true
    }).expect("Could not listen for Interfaces Removed signal");

    loop { conn.process(Duration::from_millis(1000))?; }
}

fn notify_mounted(path: &String) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_session()?;
    let notification = conn.with_proxy("org.freedesktop.Notifications", "/org/freedesktop/Notifications", Duration::from_millis(5000));

    let hints: HashMap<String, Variant<&str>> = HashMap::new();

    let replaces: u32 = 0;
    let timeout: i32 = -1;
    let (_notification_id,): (u32,) = notification.method_call(
        "org.freedesktop.Notifications",
        "Notify",
        (
            "mounter",
            replaces,
            "",
            "Filesystem mounted",
            path,
            Array::new(&[""]),
            Dict::new(&hints),
            timeout
        )
    ).unwrap();

    Ok(())
}
