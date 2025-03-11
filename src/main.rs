use crawler::modules::fetcher::client::Fetcher;
use crawler::{config::load_config, thread::ThreadPool};
use reqwest::{self, Client};
use select::document::Document;
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
    tokio::task::spawn(async move {
        let mut pool = ThreadPool::new();
        pool.execute(|| println!("Hello from thread!")).await;
        drop(pool);
        dbg!("pool dropped");
    });
    let res = reqwest::get("https://example.com").await?;
    let config = load_config("config.json")?;
    dbg!(&config);

    let document = Document::from_read(res.text().await?.as_bytes()).unwrap();
    // dbg!(&document);

    dbg!(test_fetch("https://webflow.com/made-in-webflow/links").await?);

    Ok(())
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_fn() {
        let result = test_fetch("https://webflow.com/made-in-webflow/links").await;
        assert!(result.is_ok());
    }
}
