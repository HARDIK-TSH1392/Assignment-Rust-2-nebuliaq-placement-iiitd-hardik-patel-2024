use tokio::net::TcpListener;
use tokio::io::AsyncReadExt; // Import the necessary trait for `read`
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use std::sync::Arc;

struct LogServer {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl LogServer {
    async fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
        }
    }

    async fn run(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();
        let buffer = Arc::clone(&self.buffer);

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                Self::flush_buffer(&buffer).await;
            }
        });

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let buffer = Arc::clone(&self.buffer);

            tokio::spawn(async move {
                let mut buf = [0; 1024];
                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                            let mut buffer_guard = buffer.lock().await;
                            buffer_guard.push(msg);

                            if buffer_guard.len() >= 100 {
                                // Pass the Arc<Mutex<Vec<String>>> instead of the guard
                                Self::flush_buffer(&buffer).await;
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
            });
        }
    }

    async fn flush_buffer(buffer: &Arc<Mutex<Vec<String>>>) {
        let mut buffer_guard = buffer.lock().await;

        if !buffer_guard.is_empty() {
            // Simulate sending to a destination server
            println!("Sending {} log messages to the destination server.", buffer_guard.len());
            buffer_guard.clear();
        }
    }
}

#[tokio::main]
async fn main() {
    let server = LogServer::new().await;
    server.run("127.0.0.1:8080").await;
}