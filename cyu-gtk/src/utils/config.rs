use super::constants::APP_SHORT_ID;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::RwLock;

pub const CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    let dirs =
        xdg::BaseDirectories::with_prefix(APP_SHORT_ID).expect("failed to get xdg directories");
    let file_path = dirs
        .place_config_file("config.toml")
        .expect("failed to resolve config file path");
    if !file_path.exists() {
        std::fs::File::create(&file_path).expect("failed to create config file");
    }
    let config = Config::from_file(file_path).expect("failed to load config file");
    RwLock::new(config)
});

const fn default_save_credentials() -> bool {
    false
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ConfigContent {
    #[serde(default = "default_save_credentials")]
    pub save_credentials: bool,
}

impl ConfigContent {
    fn from_file<P>(file_path: P) -> Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let content = std::fs::read_to_string(file_path)?;
        let content = toml::from_str(&content)?;
        Ok(content)
    }
}

pub struct Config {
    content: ConfigContent,
    file_path: std::path::PathBuf,
}

impl Config {
    fn from_file(path: std::path::PathBuf) -> Result<Self> {
        let content = ConfigContent::from_file(&path)?;
        Ok(Self {
            content,
            file_path: path,
        })
    }

    pub fn save_credentials(&self) -> bool {
        self.content.save_credentials
    }

    pub fn set_save_credentials(&mut self, save_credentials: bool) {
        self.content.save_credentials = save_credentials;
        let Ok(content) = toml::to_string(&self.content) else {
            eprintln!("Failed to serialize config file");
            return;
        };
        let result = std::fs::write(&self.file_path, content);
        if let Err(e) = result {
            eprintln!("Failed to write config file: {}", e);
        }
    }
}
