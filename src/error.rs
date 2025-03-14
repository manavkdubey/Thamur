use std::fmt;
use std::sync::{MutexGuard, PoisonError, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;
use tracing::error;

// Custom error enum for URL validation failures
#[derive(Debug, Error)]
pub enum CrawlerError {
    #[error("Invalid URL scheme. Only http and https are allowed")]
    InvalidScheme,
    #[error("Invalid domain. Domain must be a valid hostname")]
    InvalidDomain,
    #[error("Invalid path. Path must be a valid URL path")]
    InvalidPath,
    #[error("Invalid query parameter. Query parameters must be valid key-value pairs")]
    InvalidQueryParameter,
    #[error("Invalid URL fragment. Fragment must be a valid string")]
    InvalidFragment,
    #[error("Failed to parse URL: {0}")]
    UrlParseError(url::ParseError),
    #[error("Failed to make HTTP request: {0}")]
    HyperError(reqwest::Error),
    #[error("HTTP request failed with status code: {0}")]
    HttpError(reqwest::StatusCode),
    #[error("I/O error: {0}")]
    IoError(std::io::Error),
    #[error("Mutex lock poisoned")]
    MutexPoisonError,
    #[error("RwLock read lock poisoned")]
    RwLockReadPoisonError,
    #[error("RwLock write lock poisoned")]
    RwLockWritePoisonError,
    #[error("Other error: {0}")]
    Other(String),
    #[error("Rate limit exceeded")]
    RateLimitError(u64),
    #[error("No token available")]
    NoTokenAvailable,
}

impl From<url::ParseError> for CrawlerError {
    fn from(e: url::ParseError) -> Self {
        CrawlerError::UrlParseError(e)
    }
}

impl From<reqwest::Error> for CrawlerError {
    fn from(e: reqwest::Error) -> Self {
        error!("Failed to make HTTP request: {}", e);
        CrawlerError::HyperError(e)
    }
}
impl<T> From<PoisonError<MutexGuard<'_, T>>> for CrawlerError {
    fn from(_: PoisonError<MutexGuard<'_, T>>) -> Self {
        CrawlerError::MutexPoisonError
    }
}

impl<T> From<PoisonError<RwLockReadGuard<'_, T>>> for CrawlerError {
    fn from(_: PoisonError<RwLockReadGuard<'_, T>>) -> Self {
        CrawlerError::RwLockReadPoisonError
    }
}

impl<T> From<PoisonError<RwLockWriteGuard<'_, T>>> for CrawlerError {
    fn from(_: PoisonError<RwLockWriteGuard<'_, T>>) -> Self {
        CrawlerError::RwLockWritePoisonError
    }
}
