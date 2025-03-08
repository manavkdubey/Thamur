use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let res = reqwest::get("https://example.com").await?;

    let document = Document::from_read(res.text().await?.as_bytes()).unwrap();
    // dbg!(&document);

    for node in document.find(Name("a")) {
        println!("Found link: {}", node.attr("href").unwrap());
    }

    Ok(())
}
