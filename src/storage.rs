use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::fmt::format;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub output_path: String,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataEntry {
    pub url: String,
    pub status_code: i32,
    pub content_type: Option<String>,
    pub title: Option<String>,
    pub crawled_at: DateTime<Utc>,
}
pub struct Storage {
    config: StorageConfig,
}
pub fn get_storage_config_path() -> String {
    "storage_config.json".to_string()
}

impl StorageConfig {
    pub fn from(
        config_path: &str,
        file_name: Option<&str>,
    ) -> Result<StorageConfig, Box<dyn std::error::Error>> {
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut config: StorageConfig = serde_json::from_str(&contents)?;
        if let Some(file_name) = file_name {
            config.file_name = format!("{file_name}.json");
        }
        Ok(config)
    }
}

impl Storage {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    pub fn save_data(&self, data: &DataEntry) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(&self.config.output_path);
        let file_path = path.join(&self.config.file_name);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path.clone())?;

        file.write_all(serde_json::to_string(data)?.as_bytes())?;
        file.write_all(b"\n")?;

        tracing::info!("Data successfully saved to: {}", file_path.display());
        Ok(())
    }
    pub fn get_data(&self) -> Result<Vec<DataEntry>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.config.output_path);
        let file_path = path.join(&self.config.file_name);

        let mut file = File::open(file_path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data: Vec<DataEntry> = serde_json::from_str(&contents)?;
        Ok(data)
    }
    pub fn get_data_by_url(
        &self,
        url: &str,
    ) -> Result<Option<DataEntry>, Box<dyn std::error::Error>> {
        let data = self.get_data()?;
        Ok(data.into_iter().find(|entry| entry.url == url))
    }
}
