use crate::notifications::{Notifiable, Notification};

pub enum Notice<'a> {
    NewFilesystem(&'a str),
    NewEncrypted(&'a str),
    MountSuccess(&'a str),
    MountFail(&'a str),
    UnmountSuccess(&'a str),
    DecryptSuccess(&'a str),
    DecryptFail(&'a str)
}

impl<'a> Notifiable for Notice<'a> {
    fn as_notification(self) -> Notification {
        let mut notification = Notification::default();

        match self {
            Self::NewFilesystem(msg) => {
                notification.set_summary("New filesystem found");
                notification.set_body(msg);
            },
            Self::NewEncrypted(msg) => {
                notification.set_summary("New encrypted device found");
                notification.set_body(msg);
            },
            Self::MountSuccess(msg) => {
                notification.set_summary("Filesystem mounted");
                notification.set_body(msg);
            },
            Self::MountFail(msg) => {
                notification.set_summary("Failed to mount");
                notification.set_body(msg);
            },
            Self::UnmountSuccess(msg) => {
                notification.set_summary("Filesystem unmounted");
                notification.set_body(msg);
            },
            Self::DecryptSuccess(msg) => {
                notification.set_summary("Device decrypted");
                notification.set_body(&format!("Cleartext device: {}", msg));
            },
            Self::DecryptFail(msg) => {
                notification.set_summary("Failed to decrypt");
                notification.set_body(msg);
            }
        };

        notification
    }
}
