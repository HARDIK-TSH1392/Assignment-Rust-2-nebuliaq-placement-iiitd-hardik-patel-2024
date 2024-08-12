use std::process::Command;
use std::fs;
use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use std::sync::Arc;
use tokio::signal;
use chrono::Utc;

struct LogServer {
    buffer: Arc<Mutex<Vec<String>>>,
    quickwit_path: String,
}

impl LogServer {
    async fn new(quickwit_path: String) -> Self {
        println!("Initializing server...");
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
            quickwit_path,
        }
    }

    async fn run(&self, addr: &str) {
        println!("Binding to address: {}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        let buffer = Arc::clone(&self.buffer);

        let buffer_clone = Arc::clone(&self.buffer);
        let quickwit_path_clone = self.quickwit_path.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                println!("Flushing buffer due to time interval...");
                Self::flush_buffer(&buffer_clone, &quickwit_path_clone).await;
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
                    let quickwit_path_clone = self.quickwit_path.clone();

                    tokio::spawn(async move {
                        let mut buf = vec![0; 1024];
                        loop {
                            match socket.read(&mut buf).await {
                                Ok(0) => break,
                                Ok(n) => {
                                    let received_data = &buf[..n];
                                    println!("Received raw data: {:?}", received_data);

                                    let messages = String::from_utf8_lossy(received_data)
                                        .split('\n')
                                        .filter(|msg| !msg.is_empty())
                                        .map(|msg| msg.to_string())
                                        .collect::<Vec<String>>();

                                    for msg in messages {
                                        println!("Processing message: {}", msg);
                                        let mut buffer_guard = buffer_clone.lock().await;
                                        buffer_guard.push(msg);

                                        if buffer_guard.len() >= 100 {
                                            println!("Buffer full, flushing...");
                                            Self::flush_buffer(&buffer_clone, &quickwit_path_clone).await;
                                        }
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
                Self::flush_buffer(&buffer, &self.quickwit_path).await;
            }
        }
    }

    async fn flush_buffer(buffer: &Arc<Mutex<Vec<String>>>, quickwit_path: &str) {
        let mut buffer_guard = buffer.lock().await;
        let mut to_send = Vec::with_capacity(100);
        std::mem::swap(&mut *buffer_guard, &mut to_send);
        drop(buffer_guard);

        if !to_send.is_empty() {
            println!("Sending {} log messages to Quickwit.", to_send.len());

            // Geting the current timestamp
            let current_timestamp = Utc::now().timestamp_millis();

            // Converting log messages to JSON format without double-wrapping
            let json_docs: Vec<String> = to_send.iter().map(|msg| {
                format!(r#"{{"log_message": "{}", "timestamp": {}}}"#, msg, current_timestamp)
            }).collect();

            // Printing the documents to be indexed for debugging
            println!("Documents to be indexed:\n{}", json_docs.join("\n"));

            // Writing JSON docs to a temporary file, each on a new line
            let tmp_file = "/tmp/quickwit_bulk_data.json";
            fs::write(tmp_file, json_docs.join("\n")).expect("Unable to write temporary file");

            // Running Quickwit ingest command
            let output = Command::new(quickwit_path)
                .arg("index")
                .arg("ingest")
                .arg("--index")
                .arg("rust_log_indexer")
                .arg("--input-path")
                .arg(tmp_file)
                .arg("--force")
                .output()
                .expect("Failed to execute Quickwit command");

            if output.status.success() {
                println!("Successfully indexed {} messages to Quickwit.", to_send.len());
            } else {
                eprintln!("Failed to index messages to Quickwit. Error: {:?}", output);
            }

            // Optionally delete the temporary file after indexing
            fs::remove_file(tmp_file).expect("Failed to remove temporary file");
        }
    }
}

#[tokio::main]
async fn main() {
    let quickwit_path = "/Users/hardik/quickwit-v0.8.2/quickwit".to_string();
    let server = LogServer::new(quickwit_path).await;
    server.run("127.0.0.1:8080").await;
}
