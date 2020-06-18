use std::vec::Vec;
use std::error;
use std::fmt;
mod notification;
mod dbus_interface;
pub use notification::Notification;

type Result<T> = std::result::Result<T, NotifierError>;

pub trait Notifiable {
    fn as_notification(self) -> Notification;
}

#[derive(Debug)]
pub struct NotifierError(dbus::Error);

impl fmt::Display for NotifierError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Notifier error: {}", self.0)
    }
}

impl error::Error for NotifierError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<dbus::Error> for NotifierError {
    fn from(err: dbus::Error) -> NotifierError {
        NotifierError(err)
    }
}

#[allow(dead_code)]
pub struct Notifier {
    history: Vec<Notification>
}

impl Notifier {
    pub fn notify(notifiable: impl Notifiable) -> Result<Self> {
        let mut notification = notifiable.as_notification();
        notification.send()?;

        Ok(Notifier {
            history: vec!(notification)
        })
    }

    pub fn replace_last(&mut self, replacement: impl Notifiable) -> Result<()> {
        let mut replacement = replacement.as_notification();
        if let Some(last_notification_id) = self.history.last().and_then(|ln| ln.get_notification_id()) {
            replacement.set_replaces_id(last_notification_id);
            replacement.send()?;
            self.history.push(replacement);
        }

        Ok(())
    }
}
