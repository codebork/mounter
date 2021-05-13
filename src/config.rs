use std::fs::File;
use std::io::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Parses the config file and sets helpful defaults
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Base settings when dealing with filesystems
    #[serde(default)]
    pub settings: Settings,
    /// Mappings between UUIDs and their settings
    pub uuid: Option<HashMap<String, FsSettings>>
}

impl Config {
    /// Create default config
    pub fn new() -> Self {
        Config {
            settings: Settings::default(),
            uuid: None
        }
    }

    /// Fetches specific configuration for a UUID if any exists
    pub fn get_uuid_settings(&self, uuid: &str) -> Option<&FsSettings> {
        self.uuid.as_ref()?.get(uuid)
    }

    /// Parse configuration file and return an instance of Config
    /// with the settings specified
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

/// Generic settings when dealing with devices, can be overwritten on a per
/// filesystem basis
#[derive(Debug, Deserialize)]
// Sets all fields to default() values and
// overwrites ones that are present in toml file
#[serde(default)]
pub struct Settings {
    /// Should filesystems be mounted automatically
    pub automount: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            automount: false,
        }
    }
}

/// Filesystem specific options
#[derive(Debug, Deserialize)]
pub struct FsSettings {
    /// Should filesystem be mounted automatically
    pub automount: Option<bool>,
    /// Script to run when filesystem is mounted
    pub run: Option<String>,
    /// Password to use if filesystem is encrypted
    pub password: Option<String>,
    /// Path to keyfile to use if filesystem is encrypted
    pub keyfile: Option<String>
}
