use reqwest::{Client, StatusCode};
use std::{f32::consts::E, sync::OnceLock, time::Duration};

use crate::{
    error::CrawlerError,
    limiter::{get_rate_limiter, RateLimiter},
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
        let rate_limiter = get_rate_limiter();
        let mut retries = 0;
        let max_retries = 5;

        loop {
            match rate_limiter.check_tokens() {
                Ok(_) => break,
                Err(CrawlerError::RateLimitError(wait_time)) => {
                    if retries >= max_retries {
                        return Err(CrawlerError::RateLimitError(wait_time));
                    }
                    let backoff_time = wait_time * (retries + 1) as u64;
                    tracing::warn!(
                        "Rate limit exceeded. Retrying in {} seconds...",
                        backoff_time
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_time)).await;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }

        let response = self
            .client
            .get(url)
            .timeout(Duration::new(10, 0))
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
