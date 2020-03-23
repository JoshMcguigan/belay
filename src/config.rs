use std::fs;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub command_blacklist: Vec<String>,
}

impl Config {
    pub fn read() -> Self {
        let project_dirs =
            ProjectDirs::from("com", "cargo", "belay").expect("failed to find home directory");
        let config_path = project_dirs.config_dir().join("config.yml");

        if config_path.is_file() {
            serde_yaml::from_str::<Config>(
                &fs::read_to_string(config_path).expect("failed to read config file"),
            )
            .expect("failed to read config file as yaml")
        } else {
            fs::create_dir_all(config_path.parent().expect("config dir should have parent"))
                .expect("failed to create config directory");
            let config = Self::default();

            let config_as_string =
                serde_yaml::to_string(&config).expect("failed to convert default config to string");

            fs::File::create(&config_path).expect("failed to create config file");

            fs::write(&config_path, config_as_string.as_bytes())
                .expect("failed to write config to file");

            config
        }
    }

    /// Creates a default configuration.
    ///
    /// This is specifically not an implementation of the Default
    /// trait because we want it to be on accesible from this module.
    fn default() -> Self {
        Self {
            command_blacklist: vec![
                "apt install".into(),
                "cargo install".into(),
                "chown".into(),
                "rustup component add".into(),
            ],
        }
    }
}
