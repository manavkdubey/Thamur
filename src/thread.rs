use flume::{unbounded, Receiver, SendError, Sender};
use std::thread;
use std::time::Duration;
use tokio::task;
pub struct ThreadPool {
    workers: Vec<task::JoinHandle<()>>,
    task_sender: Sender<Box<dyn Fn() + Send + 'static>>,
}

impl ThreadPool {
    pub fn new() -> Self {
        let (task_sender, task_receiver) = unbounded();
        let mut workers = Vec::new();
        for _ in 0..4 {
            let task_receiver: Receiver<Box<dyn Fn() + Send + 'static>> = task_receiver.clone();
            let worker = task::spawn(async move {
                while let Ok(task) = task_receiver.recv_async().await {
                    task();
                }
                println!("Worker exiting...");
            });
            workers.push(worker);
        }
        Self {
            workers,
            task_sender,
        }
    }
    pub async fn execute<F>(&self, f: F) -> Result<(), SendError<Box<dyn Fn() + Send>>>
    where
        F: Fn() + Send + 'static,
    {
        self.task_sender.send_async(Box::new(f)).await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        for handle in self.workers.drain(..) {
            if let Err(e) = handle.await {
                tracing::error!("Error waiting for thread to complete: {}", e);
            }
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        let workers = std::mem::take(&mut self.workers);

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn_blocking(move || {
                for worker in workers {
                    let _ = futures::executor::block_on(worker);
                }
            });
        } else {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for worker in workers {
                    if let Err(e) = worker.await {
                        tracing::error!("Error waiting for thread to complete: {}", e);
                    }
                }
            });
        }
    }
}

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thread_pool() {
        let mut pool = ThreadPool::new();
        pool.execute(|| println!("Hello from thread!")).await;
        drop(pool);
        dbg!("pool dropped");
    }
}
