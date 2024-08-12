use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio::io::AsyncWriteExt;

async fn send_log_messages(rate: u64, addr: &str) {
    let mut stream = TcpStream::connect(addr).await.unwrap();

    let mut interval = time::interval(Duration::from_millis(rate));
    let mut count = 0;

    loop {
        interval.tick().await;
        let log_message = format!("{{\"log_message\": \"Log message {}\"}}", count);
        match stream.write_all(log_message.as_bytes()).await {
            Ok(_) => {
                println!("Sent: {}", log_message);
            }
            Err(e) => {
                println!("Failed to send message: {}. Error: {}", log_message, e);
                println!("Connection lost. Attempting to reconnect...");

                loop {
                    match TcpStream::connect(addr).await {
                        Ok(s) => {
                            println!("Reconnected to the server.");
                            stream = s;
                            break;
                        }
                        Err(e) => {
                            println!("Reconnection failed, retrying... Error: {}", e);
                            time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        }
        count += 1;
    }
}

#[tokio::main]
async fn main() {
    println!("Starting client...");
    // Sending a message every 100 milliseconds
    send_log_messages(10, "127.0.0.1:8080").await;  
}
