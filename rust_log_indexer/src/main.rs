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
        println!("Initializing server..."); 
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
        }
    }

    async fn run(&self, addr: &str) {
        println!("Binding to address: {}", addr); 
        let listener = TcpListener::bind(addr).await.unwrap();
        let buffer = Arc::clone(&self.buffer);

        // Handling the periodic flushing of the buffer
        let buffer_clone = Arc::clone(&self.buffer);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                println!("Flushing buffer due to time interval..."); 
                Self::flush_buffer(&buffer_clone).await;
            }
        });

        let shutdown_signal = signal::ctrl_c();
        tokio::select! {
            _ = async {
                loop {
                    println!("Waiting for client connection..."); 
                    let (mut socket, _) = listener.accept().await.unwrap();
                    println!("Client connected."); 
                    let buffer_clone = Arc::clone(&buffer);

                    tokio::spawn(async move {
                        let mut buf = [0; 1024];
                        loop {
                            match socket.read(&mut buf).await {
                                // Connection closed by client
                                Ok(0) => break,
                                Ok(n) => {
                                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                    println!("Received message: {}", msg); 
                                    let mut buffer_guard = buffer_clone.lock().await;
                                    buffer_guard.push(msg);

                                    if buffer_guard.len() >= 100 {
                                        println!("Buffer full, flushing...");
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
        drop(buffer_guard);

        if !to_send.is_empty() {
            // Simulate sending to a destination server
            println!("Sending {} log messages to the destination server.", to_send.len());
        }
    }
}

#[tokio::main]
async fn main() {
    let server = LogServer::new().await;
    server.run("127.0.0.1:8080").await;
}
