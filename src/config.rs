use serde_derive::Deserialize;
use serde_json;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct CrawlerConfig {
    pub user_agent: String,
    pub max_depth: u32,
    pub max_threads: u32,
    pub timeout: u64,
}

pub fn load_config(path: &str) -> Result<CrawlerConfig, Box<dyn std::error::Error>> {
    let file = File::open(Path::new(path))?;
    let config: CrawlerConfig = serde_json::from_reader(file)?;
    Ok(config)
}
