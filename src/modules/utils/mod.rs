use url::{ParseError, Url};

pub fn is_valid_url(url: &str) -> bool {
    match Url::parse(url) {
        Ok(url) => url.has_host() && url.scheme() == "http" || url.scheme() == "https",
        Err(_) => false,
    }
}
pub fn normalize_url(base_url: &str, relative_url: &str) -> String {
    if Url::parse(relative_url).is_ok() {
        relative_url.to_string()
    } else {
        format!("{}{}", base_url, relative_url)
    }
}

pub fn extract_domain(url: &str) -> Result<Option<String>, ParseError> {
    let url1 = Url::parse(url)?;
    let domain = url1.host().map(|host| host.to_string());
    Ok(domain)
}

mod tests {
    use super::*;

    #[test]
    fn test_is_valid_url() {
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://example.com"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url("example.com"));
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("http://example.com", "/path"),
            "http://example.com/path"
        );
        assert_eq!(
            normalize_url("http://example.com", "http://example.com/path"),
            "http://example.com/path"
        );
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("http://example.com").unwrap(),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain("https://example.com").unwrap(),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_domain("ftp://example.com").unwrap(),
            Some("example.com".to_string())
        );
        assert_eq!(extract_domain("example.com").ok(), None);
    }
}
