use dbus::blocking::Connection;
use dbus::arg::Variant;
use std::time::Duration;
use std::collections::HashMap;
use super::dbus_interface::OrgFreedesktopNotifications;

#[derive(Debug, Default)]
pub struct Notification {
    notification_id: Option<u32>,
    app_name: String,
    replaces_id: u32,
    app_icon: String,
    summary: String,
    body: String,
    actions: Vec<String>,
    hints: HashMap<String, Variant<std::boxed::Box<dyn dbus::arg::RefArg>>>,
    expire_timeout: i32,
}

impl Notification {
    pub fn set_app_name(&mut self, app_name: &str) {
        self.app_name = app_name.to_string();
    }

    pub fn set_replaces_id(&mut self, replaces_id: u32) {
        self.replaces_id = replaces_id;
    }

    pub fn set_app_icon(&mut self, app_icon: &str) {
        self.app_icon = app_icon.to_string();
    }

    pub fn set_summary(&mut self, summary: &str) {
        self.summary = summary.to_string();
    }

    pub fn set_body(&mut self, body: &str) {
        self.body = body.to_string();
    }

    pub fn set_expire_timeout(&mut self, expire_timeout: i32) {
        self.expire_timeout = expire_timeout;
    }
    
    pub fn get_notification_id(&self) -> Option<u32> {
        self.notification_id
    }

    pub fn get_app_name(&self) -> &str {
        &self.app_name
    }

    pub fn get_replaces_id(&self) -> u32 {
        self.replaces_id
    }

    pub fn get_app_icon(&self) -> &str {
        &self.app_icon
    }

    pub fn get_summary(&self) -> &str {
        &self.summary
    }

    pub fn get_body(&self) -> &str {
        &self.body
    }

    pub fn get_expire_timeout(&mut self, expire_timeout: i32) {
        self.expire_timeout = expire_timeout;
    }

    pub fn send(&mut self) -> Result<(), dbus::Error> {
        let conn = Connection::new_session().expect("Could not connect to session bus");
        let proxy = conn.with_proxy(
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            Duration::from_millis(5000)
        );

        proxy.notify(
            &self.app_name,
            self.replaces_id,
            &self.app_icon,
            &self.summary,
            &self.body,
            self.actions.iter().map(AsRef::as_ref).collect(),
            HashMap::new(),
            self.expire_timeout
        ).and_then(move |id| {
            self.notification_id = Some(id);
            Ok(())
        })
    }
}

