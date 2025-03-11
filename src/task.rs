use flume::unbounded;
use std::thread;
use std::time::Duration;

enum Task {
    CrawlUrl(String),
}
impl Task {
    pub fn new(url: String) -> Self {
        Task::CrawlUrl(url)
    }
    pub fn execute(&self) -> Result<(), anyhow::Error> {
        match self {
            Task::CrawlUrl(url) => {
                // Simulate processing time
                thread::sleep(Duration::from_millis(500));
                println!("Processed: {}", url);
                // Here you would implement the actual crawling logic
            }
        }
        Ok(())
    }
}

async fn worker_thread(receiver: flume::Receiver<Task>) {
    println!("Worker thread started");

    while let Ok(task) = receiver.recv_async().await {
        match task.execute() {
            Ok(_) => println!("Task completed successfully"),
            Err(e) => eprintln!("Error executing task: {}", e),
        }
    }
}
