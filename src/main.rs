use clap::{Arg, Command};
use std::io::Write;
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
    let matches = Command::new("Thamur: Rust Web Crawler")
        .version("1.0")
        .about("A multi-threaded web crawler written in Rust")
        .arg(
            Arg::new("url")
                .help("The URL to crawl")
                .required(false)
                .index(1),
        );

    let url = match matches.get_matches().get_one::<String>("url") {
        Some(url) => url.to_owned(),
        None => {
            print!("Enter URL to crawl: ");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    // Create a thread pool
    let mut pool = ThreadPool::new(1);

    pool.execute(|| async move {
        if let Err(e) = crawl_url(&url).await {
            eprintln!("Error crawling {}: {}", url, e);
        }
    })
    .await?;

    // pool.execute(|| async move {
    //     crawl_url("http://books.toscrape.com").await;
    // })
    // .await?;
    // pool.execute(|| async move {
    //     crawl_url("https://www.robotstxt.org").await;
    // })
    // .await?;

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
    for url in &urls {
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
        mark_url_processed(url.to_owned());
    }

    Ok(CrawledData {
        url: url.to_string(),
        links: urls,
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

    use httpmock::prelude::*;

    #[tokio::test]
    async fn test_crawler_with_mock_response() {
        // Start a mock HTTP server
        let server = MockServer::start();

        // Define the mock response for the expected request
        let mock = server.mock(|when, then| {
            when.method(GET).path("/");

            then.status(200)
                .header("Content-Type", "text/html; charset=UTF-8")
                .body("<html><body><a href=\"/link1\">Link 1</a></body></html>");
        });

        // Call your crawler function
        let result = crawl_url(&server.url("/")).await;

        dbg!(&result);

        // Ensure mock was hit
        mock.assert();

        // Verify the extracted links
        assert!(result.is_ok());
        let extracted_links = result.unwrap();
        assert_eq!(extracted_links.links.len(), 1);
        assert_eq!(
            extracted_links.links[0],
            format!("{}/link1", server.url(""))
        );
    }
}
