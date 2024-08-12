use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use std::sync::Arc;
use tokio::signal;

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

        // Handle the periodic flushing of the buffer
        let buffer_clone = Arc::clone(&self.buffer);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                Self::flush_buffer(&buffer_clone).await;
            }
        });

        // Graceful shutdown handling
        let shutdown_signal = signal::ctrl_c();
        tokio::select! {
            _ = async {
                loop {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    let buffer_clone = Arc::clone(&buffer);

                    tokio::spawn(async move {
                        let mut buf = [0; 1024];
                        loop {
                            match socket.read(&mut buf).await {
                                Ok(0) => break, // Connection closed by client
                                Ok(n) => {
                                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                    let mut buffer_guard = buffer_clone.lock().await;
                                    buffer_guard.push(msg);

                                    if buffer_guard.len() >= 100 {
                                        // Flush buffer when it reaches 100 messages
                                        Self::flush_buffer(&buffer_clone).await;
                                    }
                                }
                                Err(e) => {
                                    println!("Error reading from socket: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
            } => {},
            _ = shutdown_signal => {
                println!("Shutdown signal received, flushing buffer and exiting...");
                Self::flush_buffer(&buffer).await;
            }
        }
    }

    async fn flush_buffer(buffer: &Arc<Mutex<Vec<String>>>) {
        let mut buffer_guard = buffer.lock().await;
        let mut to_send = Vec::with_capacity(100);
        std::mem::swap(&mut *buffer_guard, &mut to_send);
        drop(buffer_guard); // Release the lock early

        if !to_send.is_empty() {
            // Simulate sending to a destination server
            println!("Sending {} log messages to the destination server.", to_send.len());
            // Here you would send `to_send` to your destination server
        }
    }
}

#[tokio::main]
async fn main() {
    let server = LogServer::new().await;
    server.run("127.0.0.1:8080").await;
}
