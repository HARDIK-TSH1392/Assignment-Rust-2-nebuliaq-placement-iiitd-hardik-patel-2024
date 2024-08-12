use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio::io::AsyncWriteExt;

async fn send_log_messages(rate: u64, addr: &str) {
    let mut stream = TcpStream::connect(addr).await.unwrap();

    let mut interval = time::interval(Duration::from_millis(rate));
    let mut count = 0;

    loop {
        interval.tick().await;
        // JSON format for better structure
        let log_message = format!("{{\"log_message\": \"Log message {}\"}}", count);
        match stream.write_all(log_message.as_bytes()).await {
            Ok(_) => {
                // Log the message that was sent
                println!("Sent: {}", log_message); 
            }
            Err(e) => {
                // Log the error if sending fails
                println!("Failed to send message: {}. Error: {}", log_message, e);
                println!("Connection lost. Attempting to reconnect...");

                loop {
                    match TcpStream::connect(addr).await {
                        Ok(s) => {
                            // Log successful reconnection
                            println!("Reconnected to the server.");
                            stream = s;
                            break;
                        }
                        Err(e) => {
                            // Log failed reconnection attempts
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
    // Log that the client is starting
    println!("Starting client...");
    // Sending a message every 100 milliseconds
    // Can configure the time window from the first arugement of the function
    send_log_messages(10, "127.0.0.1:8080").await; 
}
