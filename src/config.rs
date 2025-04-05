use log::info;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_shell: String,
    pub min_uid: u32,
    pub tty: u8,
    pub max_attempts: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            min_uid: 1000,
            default_shell: "/bin/bash".to_string(),
            tty: 2,
            max_attempts: 3,
        }
    }
}

pub fn setup_config(path: &str) -> Config {
    let real_path = PathBuf::from(path);
    if real_path.exists() {
        let content = fs::read_to_string(&real_path).expect("Failed to read config file");
        toml::from_str(&content).expect("Invalid config file format")
    } else {
        let config = Config::default();
        if let Some(parent) = real_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let toml = toml::to_string_pretty(&config).unwrap();
        let mut file = fs::File::create(&real_path).expect("Failed to create config file");
        file.write_all(toml.as_bytes()).unwrap();
        info!("Created default config at {}", real_path.display());
        config
    }
}
