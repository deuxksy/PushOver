use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub profiles: Vec<Profile>,
    pub default_profile: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub user_key: String,
    #[serde(default)]
    pub pushover_token: Option<String>,
    #[serde(default)]
    pub worker_token: Option<String>,
    #[serde(default)]
    pub api_token: Option<String>,
    #[serde(default)]
    pub api_endpoint: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find config directory"))?
            .join("pushover");

        let config_path = config_dir.join("config.toml");

        if !config_path.exists() {
            fs::create_dir_all(&config_dir)?;
            let default_config = Config {
                profiles: vec![],
                default_profile: None,
            };
            default_config.save(&config_path)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_default_profile(&self) -> Option<&Profile> {
        let name = self.default_profile.as_ref()?;
        self.profiles.iter().find(|p| &p.name == name)
    }
}
