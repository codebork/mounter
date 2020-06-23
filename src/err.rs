use std::fmt;
use std::error;
use crate::notifications;

#[derive(Debug)]
pub enum MounterError {
    NoKeyProvided,
    UnlockFailed(dbus::Error),
    UnreadableKeyFile(std::io::Error),
    NotifierError(notifications::NotifierError),
}

impl fmt::Display for MounterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoKeyProvided => write!(f, "No key or password provided"),
            Self::UnlockFailed(e) => write!(f, "Unlock Failed: {}", e.message().unwrap_or("D-Bus error")),
            Self::UnreadableKeyFile(e) => write!(f, "Couldn't read provided keyfile: {}", e),
            Self::NotifierError(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for MounterError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NoKeyProvided => None,
            Self::UnlockFailed(e) => Some(e),
            Self::UnreadableKeyFile(e) => Some(e),
            Self::NotifierError(e) => Some(e),
        }
    }
}

impl From<notifications::NotifierError> for MounterError {
    fn from(err: notifications::NotifierError) -> MounterError {
        MounterError::NotifierError(err)
    }
}
