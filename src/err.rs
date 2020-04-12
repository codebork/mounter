use std::fmt;
use std::error;

#[derive(Debug)]
pub enum MounterError {
    NoKeyProvided,
    UnlockFailed(dbus::Error),
    UnreadableKeyFile(std::io::Error),
}

impl fmt::Display for MounterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoKeyProvided => write!(f, "Error beep boop"),
            Self::UnlockFailed(e) => write!(f, "Unlock Failed: {}", e.message().unwrap_or("D-Bus error")),
            Self::UnreadableKeyFile(e) => write!(f, "Couldn't read provided keyfile: {}", e),
        }
    }
}

impl error::Error for MounterError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::NoKeyProvided => None,
            Self::UnlockFailed(e) => Some(e),
            Self::UnreadableKeyFile(e) => Some(e),
        }
    }
}

