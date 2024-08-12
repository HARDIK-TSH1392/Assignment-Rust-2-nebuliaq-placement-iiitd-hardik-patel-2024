use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use tokio::time::{self, Duration};
use std::sync::Arc;
use elasticsearch::{Elasticsearch, BulkParts};
use elasticsearch::http::transport::Transport;
use serde::Serialize;
use serde_json::json;
use std::error::Error;

#[derive(Serialize)]
struct LogMessage {
    message: String,
}

async fn create_elasticsearch_client() -> Result<Elasticsearch, Box<dyn Error>> {
    let transport = Transport::single_node("http://localhost:9200")?;
    let client = Elasticsearch::new(transport);
    Ok(client)
}

struct LogServer {
    buffer: Arc<Mutex<Vec<String>>>,
    es_client: Elasticsearch,
}

impl LogServer {
    async fn new() -> Self {
        let es_client = create_elasticsearch_client().await.unwrap();

        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(100))),
            es_client,
        }
    }

    async fn run(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.unwrap();
        let buffer = Arc::clone(&self.buffer);
        let es_client_clone = self.es_client.clone();

        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;
                Self::flush_buffer(&buffer, &es_client_clone).await;
            }
        });

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let buffer = Arc::clone(&self.buffer);
            let es_client_clone = self.es_client.clone();

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
                                Self::flush_buffer(&buffer, &es_client_clone).await;
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

    async fn flush_buffer(buffer: &Arc<Mutex<Vec<String>>>, es_client: &Elasticsearch) {
        let mut buffer_guard = buffer.lock().await;

        if !buffer_guard.is_empty() {
            let bulk_ops = buffer_guard.drain(..)
                .flat_map(|message| {
                    let log_message = LogMessage { message };
                    let index_op = json!({ "index": { "_index": "logs" } });
                    let doc = serde_json::to_string(&log_message).unwrap();

                    vec![index_op.to_string(), doc]
                })
                .collect::<Vec<_>>();

            let response = es_client.bulk(BulkParts::Index("logs"))
                .body(bulk_ops)
                .send()
                .await;

            match response {
                Ok(response) => {
                    println!("Indexed {} log messages to Elasticsearch.", response.status_code());
                }
                Err(err) => {
                    eprintln!("Failed to index log messages: {:?}", err);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let server = LogServer::new().await;
    server.run("127.0.0.1:8080").await;
}
