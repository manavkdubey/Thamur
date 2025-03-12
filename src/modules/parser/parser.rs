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

pub fn normalize_url(base_url: &str, relative_url: &str) -> Result<url::Url, url::ParseError> {
    let base = url::Url::parse(base_url)?;
    let normalized = match url::Url::parse(relative_url).map(|r| r.host().map(|h| h.to_string())) {
        Ok(_) => url::Url::parse(relative_url)?,
        Err(_) => base.join(relative_url)?,
    };
    let validator = UrlValidator::new();
    if validator.is_valid(&normalized) {
        Ok(normalized)
    } else {
        Err(url::ParseError::RelativeUrlWithoutBase)
    }
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
        let expected = vec!["https://example.com/", "https://example.com/about"];
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
            "https://example.com/absolute",
            "http://test.com/relative",
            "http://example.com/page",
            "http://test.com/invalid-url",
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
        let expected_url = url::Url::parse("http://test.com/relative").unwrap();
        assert_eq!(normalize_url(base_url, relative_url).unwrap(), expected_url);
    }
    #[test]
    fn test_normalize_url_with_scheme() {
        let base_url = "http://test.com/normalize";
        let relative_url = "https://example.com/absolute";
        let expected_url = url::Url::parse("https://example.com/absolute").unwrap();
        assert_eq!(normalize_url(base_url, relative_url).unwrap(), expected_url);
    }
    #[test]
    fn test_normalize_url_with_query() {
        let base_url = "http://test.com/normalize";
        let relative_url = "/relative?query=param";
        let expected_url = url::Url::parse("http://test.com/relative?query=param").unwrap();
        assert_eq!(normalize_url(base_url, relative_url).unwrap(), expected_url);
    }
    #[test]
    fn test_normalize_url_with_fragment() {
        let base_url = "http://example.com/some/path/";
        let relative_url = "../example.html";
        let expected_url = url::Url::parse("http://example.com/some/example.html").unwrap();
        assert_eq!(normalize_url(base_url, relative_url).unwrap(), expected_url);
    }
    #[test]
    fn test_url_normalizer() {
        // Test cases
        test_normalizer(
            "http://example.com/current/page.html",
            "example.html",
            Some("http://example.com/current/example.html"),
        );

        test_normalizer(
            "http://example.com/current/",
            "../example.html",
            Some("http://example.com/example.html"),
        );

        test_normalizer(
            "http://example.com/",
            "/page",
            Some("http://example.com/page"),
        );

        test_normalizer(
            "http://example.com/",
            "http://example.com/absolute",
            Some("http://example.com/absolute"),
        );

        test_normalizer(
            "http://example.com/",
            "//example.com/protocol-relative",
            Some("http://example.com/protocol-relative"),
        );

        test_normalizer(
            "http://example.com/",
            "./relative",
            Some("http://example.com/relative"),
        );

        test_normalizer(
            "http://example.com/current/",
            "../parent",
            Some("http://example.com/parent"),
        );

        test_normalizer(
            "http://example.com/current/",
            "../../parent",
            Some("http://example.com/parent"),
        );

        test_normalizer(
            "http://example.com/current/",
            "../../../parent",
            Some("http://example.com/parent"),
        );

        test_normalizer(
            "http://example.com/current/",
            "http://example.com/absolute",
            Some("http://example.com/absolute"),
        );

        test_normalizer("http://example.com/current/", "invalid://invalid", None);
    }

    fn test_normalizer(base_url: &str, relative_url: &str, expected: Option<&str>) {
        let normalized = normalize_url(base_url, relative_url);
        match expected {
            Some(expected) => {
                assert_eq!(normalized.as_ref().map(|s| s.as_str()).ok(), Some(expected))
            }
            None => assert!(normalized.is_err()),
        }
    }
}
