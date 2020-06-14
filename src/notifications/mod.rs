use dbus::blocking::Connection;
use dbus::arg::Variant;
use std::time::Duration;
use std::collections::HashMap;
use std::vec::Vec;
pub mod interface;
use crate::notifications::interface::OrgFreedesktopNotifications;

pub trait Notifiable {
    fn as_notification(self) -> Notification;
}

#[allow(dead_code)]
pub struct Notifier {
    history: Vec<Notification>
}

impl Notifier {
    pub fn notify(notifiable: impl Notifiable) -> Result<Self, dbus::Error> {
        let mut notification = notifiable.as_notification();
        notification.send()?;

        Ok(Notifier {
            history: vec!(notification)
        })
    }

    pub fn replace_last(&mut self, replacement: impl Notifiable) -> Result<(), dbus::Error> {
        let mut replacement = replacement.as_notification();
        if let Some(last_notification) = self.history.last() {
            replacement.replaces_id = last_notification.notification_id;
            replacement.send()?;
            self.history.push(replacement);
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Notification {
    notification_id: u32,
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

    pub fn get_app_name(&self) -> &str {
        &self.app_name
    }

    pub fn get_replaces_id(&mut self) -> u32 {
        self.replaces_id
    }

    pub fn get_app_icon(&mut self) -> &str {
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
        let notifier = conn.with_proxy(
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            Duration::from_millis(5000)
        );

        notifier.notify(
            &self.app_name,
            self.replaces_id,
            &self.app_icon,
            &self.summary,
            &self.body,
            self.actions.iter().map(AsRef::as_ref).collect(),
            HashMap::new(),
            self.expire_timeout
        ).and_then(move |id| {
            self.notification_id = id;
            Ok(())
        })
    }
}

