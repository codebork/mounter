use std::fs::File;
use std::io::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    uuid: Option<HashMap<String, FsSettings>>
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: Settings::default(),
            uuid: None
        }
    }

    pub fn get_uuid_settings(&self, uuid: &str) -> Option<&FsSettings> {
        self.uuid.as_ref()?.get(uuid)
    }

    pub fn parse(path: &Path) -> Self {
        // If the file can be read then parse it otherwise
        // return an empty config file
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();

            // Read config file into string and convert into Config struct
            file.read_to_string(&mut contents).expect("Could not read file");
            toml::from_str(contents.as_str()).unwrap()
        } else {
            eprintln!("Could not read config file: {:?}", path);
            Config::new()
        }
    }
}


#[derive(Debug, Deserialize)]
// Sets all fields to default() values and
// overwrites ones that are present in toml file
#[serde(default)]
pub struct Settings {
    pub automount: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            automount: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FsSettings {
    pub automount: Option<bool>,
    pub command: Option<String>,
    pub password: Option<String>,
    pub keyfile: Option<String>
}
