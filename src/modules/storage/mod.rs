pub mod state;

// Module for storing crawled data
pub mod storage {
    pub fn store_urls(urls: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual storage logic
        Ok(())
    }
}
