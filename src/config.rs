// /src/config.rs   - Application configuration management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::error::Error;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Host {
    pub name: String,
    pub hostname: String,
    pub username: String,
    pub port: u16,
    pub use_key_auth: bool,
    pub key_path: Option<String>,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            name: "Raspberry Pi".to_string(),
            hostname: "raspberrypi.local".to_string(),
            username: "pi".to_string(),
            port: 22,
            use_key_auth: true,
            key_path: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub window_width: i32,
    pub window_height: i32,
    pub default_local_dir: String,
    pub hosts: Vec<Host>,
    pub last_used_host_index: usize,
    pub image_formats: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_width: 900,
            window_height: 700,
            default_local_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy()
                .to_string(),
            hosts: vec![Host::default()],
            last_used_host_index: 0,
            image_formats: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "tiff".to_string(),
                "webp".to_string(),
            ],
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let config_str = fs::read_to_string(&config_path)?;
        let config = serde_json::from_str(&config_str)?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::get_config_path()?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_str)?;
        
        Ok(())
    }
    
    /// Get the path to the configuration file
    fn get_config_path() -> Result<PathBuf, io::Error> {
        let proj_dirs = ProjectDirs::from("com", "PiImageProcessor", "piimgproc")
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine config directory"
            ))?;
            
        let config_dir = proj_dirs.config_dir();
        Ok(config_dir.join("config.json"))
    }
}