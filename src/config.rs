pub struct Config {
    pub command_blacklist: Vec<String>,
}

impl Config {
    pub fn read() -> Self {
        // TODO read from file system, if it doesn't exist write default to file system
        Self::default()
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
