use idna::domain_to_ascii;
use regex::Regex;
use url::{ParseError, Url};

use crate::error::CrawlerError;
const MAX_URL_LENGTH: usize = 2048;

pub struct UrlValidator {
    scheme_re: Regex,
    domain_re: Regex,
    path_re: Regex,
    query_re: Regex,
    fragment_re: Regex,
}

impl UrlValidator {
    pub fn new() -> Self {
        let scheme_re = Regex::new(r"^https?$").unwrap();
        let domain_re =
            Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z]{2,})+$").unwrap();
        let path_re = Regex::new(r"^/[a-zA-Z0-9/._%-]*$").unwrap();
        let query_re = Regex::new(r"^[a-zA-Z0-9._~-]+(=[a-zA-Z0-9._~%-]*)?$").unwrap(); // Only valid characters allowed
        let fragment_re = Regex::new(r"^[a-zA-Z0-9._%~-]*$").unwrap(); // Only valid characters allowed in fragment

        Self {
            scheme_re,
            domain_re,
            path_re,
            query_re,
            fragment_re,
        }
    }

    pub fn is_valid(&self, url: &Url) -> bool {
        if url.as_str().len() > MAX_URL_LENGTH {
            return false;
        }
        // Validate scheme
        if !self.scheme_re.is_match(url.scheme()) {
            return false;
        }

        // Convert IDN to ASCII
        let ascii_domain = match domain_to_ascii(url.host_str().unwrap_or("")) {
            Ok(d) => d,
            Err(_) => return false,
        };

        // Validate domain
        if !self.domain_re.is_match(&ascii_domain) || ascii_domain.contains("..") {
            return false;
        }
        if let Some(port) = url.port() {
            if (url.scheme() == "http" && port == 80) || (url.scheme() == "https" && port == 443) {
                return false; // Should not explicitly specify default ports
            }
        }

        // Validate path
        let path = url.path();

        if !self.path_re.is_match(path) {
            return false;
        }

        if !path.starts_with("/") && (url.path().starts_with("..") || url.path().contains("../")) {
            return false;
        }
        if let Some(query) = url.query() {
            for param in query.split('&') {
                if !self.query_re.is_match(param) {
                    return false;
                }
            }
        }
        if let Some(fragment) = url.fragment() {
            if !self.fragment_re.is_match(fragment) {
                return false;
            }
        }

        true
    }
    pub fn remove_url_fragment(&self, url: &str) -> Result<Url, CrawlerError> {
        let mut url = Url::parse(url).map_err(|e| CrawlerError::from(e))?;
        url.set_fragment(None);
        Ok(url)
    }
    pub fn is_valid_path(&self, path: &str) -> bool {
        if path.is_empty() {
            return false;
        }

        if path.starts_with('/') || path.contains("://") {
            return false;
        }

        let invalid_chars = [' ', '\\', '"', '<', '>', '`', '^', '{', '}', '|'];

        if path.chars().any(|c| invalid_chars.contains(&c)) {
            return false;
        }

        let valid_chars = |c: char| c.is_alphanumeric() || "-_.~/".contains(c);

        if !path.chars().all(valid_chars) {
            return false;
        }

        true
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_is_valid() {
        let validator = UrlValidator::new();
        let valid_url = Url::parse("https://webflow.com/made-in-webflow/links").unwrap();
        let invalid_url = Url::parse("ftp://example.com").unwrap();

        assert!(validator.is_valid(&valid_url));
        assert!(!validator.is_valid(&invalid_url));
    }
    #[test]
    fn test_is_valid_2() {
        let validator = UrlValidator::new();
        let valid_url = Url::parse("https://example.com").unwrap();
        let invalid_url = Url::parse("invalid://example").unwrap();

        assert!(validator.is_valid(&valid_url));
        assert!(!validator.is_valid(&invalid_url));
    }

    #[test]
    fn test_valid_domain() {
        let validator = UrlValidator::new();
        assert!(validator.is_valid(&Url::parse("http://example.com").unwrap()));
        assert!(validator.is_valid(&Url::parse("http://sub.example.com").unwrap()));
        assert!(validator.is_valid(&Url::parse("http://example.co.uk").unwrap()));
        assert!(!validator.is_valid(&Url::parse("http://example").unwrap()));
        assert!(!validator.is_valid(&Url::parse("http://.com").unwrap()));
        assert!(validator.is_valid(&Url::parse("https://example.com/path?a=1&b=2").unwrap()));
    }
    #[test]
    fn test_remove_url_fragment() {
        let validator = UrlValidator::new();
        let cleaned_url = validator
            .remove_url_fragment("http://example.com/page#section2")
            .unwrap();
        assert_eq!(cleaned_url, Url::parse("http://example.com/page").unwrap());
    }
}
