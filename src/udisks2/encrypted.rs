use dbus::blocking::Connection;
use std::collections::HashMap;
use dbus::arg::Variant;
use dbus::strings::Path;
use crate::udisks2::block;
use crate::err::MounterError;

pub struct Encrypted {
    pub device: block::Block
}

impl Encrypted {
    pub fn unlock(&self, keyfile: &Option<String>, password: &Option<String>) -> Result<String, MounterError> {
        let conn = Connection::new_system().expect("Could not connect to system bus");
        let proxy = conn.with_proxy("org.freedesktop.UDisks2", &self.device.object_path, std::time::Duration::from_millis(5000));
        let mut options: HashMap<String, Variant<Vec<u8>>> = HashMap::new();

        if let Some(keyfile_path) = keyfile {
            std::fs::read(keyfile_path).map_or_else(
                |e| {
                    Err(MounterError::UnreadableKeyFile(e))
                },
                |bytes| {
                    options.insert("keyfile_contents".to_string(), Variant(bytes));

                    proxy.method_call(
                        "org.freedesktop.UDisks2.Encrypted",
                        "Unlock",
                        (String::new(), options,)
                    ).map_or_else(
                        |e| {
                            Err(MounterError::UnlockFailed(e))
                        },
                        |(object_path,): (Path,)| {
                            Ok(object_path.to_string())
                        }
                    )
                }
            )
        } else if let Some(password) = password {
            proxy.method_call(
                "org.freedesktop.UDisks2.Encrypted",
                "Unlock",
                (password, options,)
            ).map_or_else(
                |e| {
                    Err(MounterError::UnlockFailed(e))
                },
                |(object_path,): (Path,)| {
                    Ok(object_path.to_string())
                }
            )
        } else {
            Err(MounterError::NoKeyProvided)
        }
    }
}

