# 🗿 Thamur: The Rust Web Crawler

## 📖 Overview
This is a **multi-threaded, async web crawler** built in Rust. It efficiently fetches and parses web pages while respecting `robots.txt` rules, handling rate limits, and storing crawled data. The project is modular, making it **scalable and extensible**.

## 🎯 Features
✅ **Asynchronous Crawling** using `tokio`
✅ **Concurrency Management** with a custom thread pool (`flume` + `tokio::task`)
✅ **Rate Limiting** via a **token bucket** strategy
✅ **Respects robots.txt** and obeys crawl restrictions
✅ **Efficient URL Validation** using regex
✅ **Stores crawled data** in JSON format
✅ **Error Handling** with `thiserror`
✅ **Modular Design** for extensibility

## 📂 Project Structure
```
├── src/
│   ├── main.rs          # Entry point, manages tasks
│   ├── client.rs        # Fetches web pages (reqwest)
│   ├── parser.rs        # Extracts links from HTML (select.rs)
│   ├── robot.rs         # Parses robots.txt
│   ├── limiter.rs       # Implements rate limiting
│   ├── storage.rs       # Stores crawled data in JSON
│   ├── config.rs        # Loads config settings
│   ├── thread.rs        # Custom thread pool
│   ├── task.rs          # Task execution
│   ├── state.rs         # Manages visited URLs & queue
│   ├── validator.rs     # URL validation with regex
│   ├── error.rs         # Centralized error handling
│   ├── lib.rs           # Module declarations
│   └── modules/
│       ├── fetcher/
│       ├── parser/
│       ├── storage/
│       ├── utils/
│       ├── thread/
│       ├── validator/
│       └── ...
├── Cargo.toml           # Dependencies & metadata
└── README.md            # Project documentation
```

## ⚙️ Installation
Ensure you have **Rust** installed (via `rustup`).
```sh
# Clone this repository
git clone https://github.com/manavkdubey/Thamur.git
cd rust-web-crawler

# Build the project
cargo build --release
```

## 🚀 Usage
Run the crawler with:
```sh
cargo run --release
```
Or specify a URL to crawl:
```sh
cargo run --release -- http://example.com
```

## 🛠️ Configuration
Modify `config.json`:
```json
{
  "user_agent": "MyCrawler/1.0",
  "max_depth": 3,
  "max_threads": 5,
  "timeout": 10
}
```

## 🏗️ Contributing
1. **Fork** this repo
2. **Clone** your fork
3. **Create a new branch**
4. **Commit & push** changes
5. **Open a pull request** 🚀

## 📜 License
This project is licensed under the **MIT License**.

---
Happy Crawling! 🕷️🚀
