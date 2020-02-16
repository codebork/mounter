use dbus::strings::Path;
use dbus::arg::{Variant, RefArg};
use crate::udisks2::Udisks2InterfacesAndProps;

#[derive(Clone, Debug, Default)]
pub struct Drive {
    pub object_path: Path<'static>,
    pub removable: bool
}

impl Drive {
    pub fn new(object_path: &Path<'static>, interfaces_and_properties: &Udisks2InterfacesAndProps) -> Option<Self> {
        if let Some(drive_interface) = interfaces_and_properties.get("org.freedesktop.UDisks2.Drive") {
            let mut drive = Self {
                object_path: object_path.to_owned(),
                ..Default::default()
            };

            for (key, value) in drive_interface {
                match key.as_str() {
                    "Removable" => drive.removable = get_bool(value)?,
                    _ => ()
                }
            }
            
            Some(drive)
        } else {
            None
        }
    }
}

fn get_bool(arg: &Variant<Box<dyn RefArg>>) -> Option<bool> {
    arg.0.as_i64().and_then(|s| {
        if s != 0 { Some(true) } else { Some(false) }
    })
}
