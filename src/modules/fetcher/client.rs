use reqwest::{Client, StatusCode};
use std::time::Duration;

use crate::{
    error::CrawlerError,
    modules::storage::state::{get_global_instance, is_url_processed, mark_url_processed},
};

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

    pub async fn fetch_page(
        &self,
        url: &str,
    ) -> Result<(String, StatusCode, Option<String>), CrawlerError> {
        // if is_url_processed(url) {
        //     todo!()
        // }
        let response = self
            .client
            .get(url)
            .header("UserAgent", "Thamur/1.0")
            .send()
            .await
            .map_err(|e| CrawlerError::HyperError(e))?;

        let status = response.status();
        let headers = response.headers().clone();
        let content_type = headers.get("Content-Type");
        println!("Going to fetch page: {}", url);
        if response.status().is_success() {
            let text = response
                .text()
                .await
                .map_err(|e| CrawlerError::HyperError(e))?;
            tracing::info!("Fetched page : {}", url);
            println!("Fetched page : {}", url);
            mark_url_processed(url.to_string());
            Ok((
                text,
                status,
                content_type.map(|t| t.to_str().unwrap().to_string()),
            ))
        } else {
            tracing::warn!(
                "Failed to fetch page : {} with status {}",
                url,
                response.status()
            );
            println!(
                "Failed to fetch page : {} with status {}",
                url,
                response.status()
            );
            Err(CrawlerError::HttpError(response.status()))
        }
    }
}
