use std::sync::Arc;

use chrono::{DateTime, Utc};
use crawler::modules::fetcher::client::Fetcher;
use crawler::modules::parser::parser::parse_html_links;
use crawler::modules::storage;
use crawler::modules::storage::state::{mark_url_processed, SharedState};
use crawler::storage::{get_storage_config_path, DataEntry, Storage, StorageConfig};
use crawler::{config::load_config, thread::ThreadPool};
use reqwest::{self, Client};
use select::document::Document;
use select::node::Node;
use select::predicate::Name;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CrawledData {
    url: String,
    links: Vec<String>,
}

async fn test_fetch(url: &str) -> Result<CrawledData, Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client.get(url).send().await?;

    let document = Document::from_read(res.text().await?.as_bytes());
    let links = document?
        .find(Name("a"))
        .filter_map(|node| node.attr("href").map(|href| href.to_string()))
        .collect();

    Ok(CrawledData {
        url: url.to_string(),
        links,
    })
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pool = ThreadPool::new(1);

    let _ = pool
        .execute(|| async move {
            crawl_url("http://books.toscrape.com").await;
        })
        .await;

    pool.shutdown().await;

    Ok(())
}

async fn add_url(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let state = storage::state::get_global_instance();
    state.add_url(url)?;
    Ok(())
}

async fn crawl_url(url: &str) -> Result<CrawledData, Box<dyn std::error::Error>> {
    let fetcher = Fetcher::new().await?;
    let (html, status, _) = fetcher.fetch_page(url).await?;
    mark_url_processed(url.to_string());

    let urls = parse_html_links(&html, url)?;
    for url in urls {
        dbg!(&url);
        let (data, status, content) = fetcher.fetch_page(&url).await?;
        let storage = Storage::new(StorageConfig::from(
            &get_storage_config_path(),
            Some(url::Url::parse(&url)?.domain().unwrap()),
        )?);
        let data_entry = DataEntry {
            url: url.to_string(),
            status_code: status.as_u16() as i32,
            content_type: content,
            title: extract_title(&data),
            crawled_at: Utc::now(),
        };
        storage.save_data(&data_entry)?;
        mark_url_processed(url);
    }

    Ok(CrawledData {
        url: url.to_string(),
        links: Vec::new(),
    })
}
fn extract_title(html: &str) -> Option<String> {
    Document::from(html)
        .find(Name("title"))
        .next()
        .map(|node: Node| node.text())
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_fn() {
        let result = test_fetch("https://webflow.com/made-in-webflow/links").await;
        assert!(result.is_ok());
    }
}
