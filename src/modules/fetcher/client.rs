use reqwest::Client;
use std::time::Duration;

use crate::error::CrawlerError;

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub async fn new() -> Result<Self, CrawlerError> {
        let client = Client::builder()
            .timeout(Duration::new(10, 0))
            .build()
            .map_err(|e| CrawlerError::HyperError(e))?;
        Ok(Fetcher { client })
    }

    pub async fn fetch_page(&self, url: &str) -> Result<String, CrawlerError> {
        let response = self
            .client
            .get(url)
            .header("UserAgent", "Thamur/1.0")
            .send()
            .await
            .map_err(|e| CrawlerError::HyperError(e))?;

        if response.status().is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| CrawlerError::HyperError(e))?;
            tracing::info!("Fetched page : {}", url);
            Ok(text)
        } else {
            tracing::warn!(
                "Failed to fetch page : {} with status {}",
                url,
                response.status()
            );
            Err(CrawlerError::HttpError(response.status()))
        }
    }
}
