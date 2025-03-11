use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex, RwLock};

use crate::error::CrawlerError;
use crate::task::Task;

#[derive(Debug)]
pub struct SharedState {
    urls: Arc<Mutex<VecDeque<Task>>>,
    visited: Arc<RwLock<HashSet<Task>>>,
}

impl SharedState {
    pub fn new() -> Arc<Self> {
        Arc::new(SharedState {
            urls: Arc::new(Mutex::new(VecDeque::new())),
            visited: Arc::new(RwLock::new(HashSet::new())),
        })
    }
    pub fn get_handle(self: &Arc<Self>) -> Arc<Self> {
        Arc::clone(self)
    }
    pub fn add_url(&self, url: String) -> Result<(), CrawlerError> {
        let mut urls = self.urls.lock()?;
        urls.push_back(Task::CrawlUrl(url));
        Ok(())
    }

    pub fn add_visited(&self, url: String) -> Result<(), CrawlerError> {
        let mut visited = self.visited.write()?;
        visited.insert(Task::new(url));
        Ok(())
    }

    pub fn is_visited(&self, url: &str) -> Result<bool, CrawlerError> {
        let visited = self.visited.read()?;
        Ok(visited.contains(&Task::new(url.to_string())))
    }

    pub fn get_visited(&self) -> Result<Vec<String>, CrawlerError> {
        let visited = self.visited.read()?;
        Ok(visited
            .iter()
            .cloned()
            .map(|v| match v {
                Task::CrawlUrl(url) => url,
            })
            .collect())
    }

    pub fn get_urls(&self) -> Result<Vec<String>, CrawlerError> {
        let urls = self.urls.lock()?;
        Ok(urls
            .iter()
            .cloned()
            .map(|v| match v {
                Task::CrawlUrl(url) => url,
            })
            .collect())
    }
}

mod tests {

    use super::*;

    #[test]
    fn test_shared_state() {
        let state = SharedState::new();
        let handle = state.get_handle();

        handle.add_url("https://example.com".to_string());
        handle.add_visited("https://example.com".to_string());

        assert!(handle.is_visited("https://example.com").unwrap_or(false));
        assert!(!handle.is_visited("https://example.org").unwrap_or(false));
    }
}
