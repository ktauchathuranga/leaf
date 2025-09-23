use anyhow::Result;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub install_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub packages_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl Config {
    pub async fn load_or_create() -> Result<Self> {
        let home = home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

        let leaf_dir = home.join(".local").join("leaf");
        let bin_dir = home.join(".local").join("bin");
        let packages_dir = leaf_dir.join("packages");
        let cache_dir = leaf_dir.join("cache");
        let config_file = leaf_dir.join("config.json");

        // Create directories
        fs::create_dir_all(&leaf_dir).await?;
        fs::create_dir_all(&bin_dir).await?;
        fs::create_dir_all(&packages_dir).await?;
        fs::create_dir_all(&cache_dir).await?;

        if config_file.exists() {
            let config_json = fs::read_to_string(&config_file).await?;
            let config: Config = serde_json::from_str(&config_json)?;
            return Ok(config);
        }

        let config = Config {
            version: "1.0.0".to_string(), // This will be updated by installer
            install_dir: leaf_dir,
            bin_dir,
            packages_dir,
            cache_dir,
        };

        // Save config
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_file, config_json).await?;

        Ok(config)
    }
}
