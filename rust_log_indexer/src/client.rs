use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio::io::AsyncWriteExt;

async fn send_log_messages(rate: u64, addr: &str) {
    let mut stream = TcpStream::connect(addr).await.unwrap();

    let mut interval = time::interval(Duration::from_millis(rate));
    let mut count = 0;

    loop {
        interval.tick().await;
        let log_message = format!("Log message {}", count);
        if let Err(_) = stream.write_all(log_message.as_bytes()).await {
            println!("Connection lost. Attempting to reconnect...");
            stream = TcpStream::connect(addr).await.unwrap();
        }
        count += 1;
    }
}

#[tokio::main]
async fn main() {
    send_log_messages(1000, "127.0.0.1:8080").await;
}