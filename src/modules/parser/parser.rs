use select::document::Document;
use select::node::Node;
use select::predicate::Name;

use crate::error::CrawlerError;
use crate::validator::UrlValidator;

pub fn parse_html_links(html: &str, base_url: &str) -> Result<Vec<String>, CrawlerError> {
    let document = Document::from(html);
    let validator = UrlValidator::new();
    let base_url = url::Url::parse(base_url)?;
    let links = document
        .find(Name("a"))
        .filter_map(|element: Node| element.attr("href"))
        .filter_map(|href| match url::Url::parse(href) {
            Ok(url) if url.scheme() == "http" || url.scheme() == "https" => {
                if validator.is_valid(&url) {
                    Some(url.to_string())
                } else {
                    None
                }
            }
            Err(_) => {
                if validator.is_valid_path(href) {
                    if let Ok(url) = base_url.join(href) {
                        if validator.is_valid(&url) {
                            Some(url.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();
    Ok(links)
}

pub fn normalize_url(base_url: &str, relative_url: &str) -> Result<String, url::ParseError> {
    let base = url::Url::parse(base_url)?;
    let absolute = base.join(relative_url)?;
    Ok(absolute.to_string())
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_html_links() {
        let html = r#"
            <html>
                <body>
                    <a href="https://example.com">Example</a>
                    <a href="/about">About</a>
                </body>
            </html>
        "#;
        let base_url = "https://example.com";
        let expected = vec![
            "https://example.com/".to_string(),
            "https://example.com/about".to_string(),
        ];
        assert_eq!(parse_html_links(html, base_url).unwrap(), expected);
    }
    #[test]
    fn test_extract_links() {
        let html = r#"
                <html>
                    <body>
                        <a href="https://example.com/absolute">Absolute URL</a>
                        <a href="/relative">Relative URL</a>
                        <a href="//example.com/page">Relative URL</a>
                        <a href="mailto:test@example.com">Mailto Link</a>
                        <a href="invalid-url">Invalid URL</a>
                    </body>
                </html>
            "#;

        let base_url = "http://test.com/extract";
        let expected_urls = vec![
            "https://example.com/absolute".to_string(),
            "http://test.com/relative".to_string(),
            "http://example.com/page".to_string(),
        ];

        let actual_urls = parse_html_links(html, base_url).unwrap();
        assert_eq!(actual_urls, expected_urls);

        // Test edge cases
        let html_with_fragment = r#"<a href="/#fragment/">Fragment</a>"#;
        assert!(parse_html_links(html_with_fragment, base_url)
            .unwrap()
            .is_empty());
    }
    #[test]
    fn test_normalize_url() {
        let base_url = "http://test.com/normalize";
        let relative_url = "/relative";
        let expected_url = "http://test.com/relative";
        assert_eq!(normalize_url(base_url, relative_url).unwrap(), expected_url);
    }
    #[test]
    fn test_normalize_url_with_scheme() {
        let base_url = "http://test.com/normalize";
        let relative_url = "https://example.com/absolute";
        let expected_url = "https://example.com/absolute";
        assert_eq!(normalize_url(base_url, relative_url).unwrap(), expected_url);
    }
}
