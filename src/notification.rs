use dbus::blocking::Connection;
use dbus::arg::Variant;
use std::time::Duration;
use std::collections::HashMap;
use std::vec::Vec;

#[derive(Debug, Default)]
pub struct Notification {
    app_name: String,
    replaces_id: u32,
    app_icon: String,
    summary: String,
    body: String,
    actions: Vec<String>,
    hints: HashMap<String, Variant<String>>,
    expire_timeout: i32
}

impl Notification {
    pub fn send(&self) -> u32 {
        let conn = Connection::new_session().expect("Could not connect to session bus");
        let notifier = conn.with_proxy(
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            Duration::from_millis(5000)
        );

        let (notification_id,): (u32,) = notifier.method_call(
            "org.freedesktop.Notifications",
            "Notify",
            (
                &self.app_name,
                &self.replaces_id,
                &self.app_icon,
                &self.summary,
                &self.body,
                &self.actions,
                &self.hints,
                &self.expire_timeout
            )
        ).expect("Could not send notification");

        notification_id
    }
}

pub fn new_filesystem(body: &str) -> Notification {
    Notification {
        summary: "New filesystem found".to_string(),
        body: body.to_string(),
        ..Default::default()
    }
}

pub fn new_encrypted(body: &str) -> Notification {
    Notification {
        summary: "New encrypted device found".to_string(),
        body: body.to_string(),
        ..Default::default()
    }
}

pub fn mounted(object_path: &str) -> Notification {
    Notification {
        summary: "Filesystem mounted".to_string(),
        body: object_path.to_string(),
        ..Default::default()
    }
}

pub fn unmounted(object_path: &str) -> Notification {
    Notification {
        summary: "Filesystem unmounted".to_string(),
        body: object_path.to_string(),
        ..Default::default()
    }
}

pub fn decrypted(object_path: &str) -> Notification {
    Notification {
        summary: "Device decrypted".to_string(),
        body: format!("Cleartext device: {}", object_path),
        ..Default::default()
    }
}

pub fn mount_failed(device: &str) -> Notification {
    Notification {
        summary: "Mount failed".to_string(),
        body: format!("Could not mount {}", device),
        ..Default::default()
    }
}

pub fn decryption_failed(device: &str) -> Notification {
    Notification {
        summary: "Encrypted device error".to_string(),
        body: device.to_string(),
        ..Default::default()
    }
}
