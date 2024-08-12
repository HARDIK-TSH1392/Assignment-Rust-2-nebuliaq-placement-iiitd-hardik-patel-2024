
# Rust Log Indexer

This project implements a TCP server that buffers incoming log messages and sends them to a destination server in batches. The server is built using the Tokio async runtime in Rust. The client simulates sending log messages to the server at a configurable rate and handles reconnecting if the connection is lost.

## Features
- **TCP Server**: Listens for incoming log messages, buffers them in batches of 100 or every 10 seconds, and sends them to a destination server.
- **Client**: Simulates sending log messages to the server at a configurable rate and reconnects automatically if the connection is lost.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (version 1.56.0 or later)
- [Cargo](https://doc.rust-lang.org/cargo/) (Rust's package manager)
- [Tokio](https://tokio.rs/) (Asynchronous runtime for Rust)

### Installation
1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd rust_log_indexer
   ```

2. Build the project:
   ```bash
   cargo build
   ```

### Running the TCP Server

To start the server, run the following command:

```bash
cargo run --bin rust_log_indexer
```

This will start the TCP server, listening on `127.0.0.1:8080` by default. The server will buffer incoming log messages and send them to a destination server either when the buffer reaches 100 messages or after 10 seconds, whichever comes first.

### Running the Client

To start the client that simulates sending log messages to the server, run the following command:

```bash
cargo run --bin client
```

This will start the client, which connects to the server at `127.0.0.1:8080` and sends log messages every 100 milliseconds by default. If the connection is lost, the client will automatically attempt to reconnect.

### Configuration

The client’s log message rate and the server's address can be configured by modifying the `send_log_messages` function in `client.rs` and the `run` function in `main.rs`.

### Shutting Down

The server listens for a shutdown signal (`Ctrl+C`). When the signal is received, it will flush any remaining messages in the buffer before shutting down gracefully.

## Approach

### Server Implementation

- **Concurrency and Buffering**: The server is implemented using the Tokio asynchronous runtime, which allows it to handle multiple client connections concurrently. The incoming log messages are buffered in an in-memory vector. This buffer is shared among tasks using an `Arc<Mutex<Vec<String>>>` to ensure thread safety and to allow multiple tasks to access the buffer concurrently.
  
- **Buffer Management**: The server buffers log messages until either 100 messages are received or 10 seconds have passed. The buffer is then flushed and the messages are sent to a destination server. The buffer is managed by swapping its contents with an empty vector, which minimizes the time the buffer is locked, improving performance.

- **Graceful Shutdown**: The server listens for a shutdown signal (`Ctrl+C`). Upon receiving this signal, the server flushes any remaining log messages in the buffer before shutting down.

### Client Implementation

- **Log Message Simulation**: The client simulates log message generation at a configurable rate, sending these messages to the server over a TCP connection. The log messages are formatted in JSON to provide a structured format.

- **Reconnection Handling**: If the connection to the server is lost, the client will attempt to reconnect automatically, logging each attempt. This ensures that the client can continue sending log messages even if the server temporarily goes offline.

### Code Quality

The code is structured to be maintainable and extendable, with clear separation of concerns. Error handling is implemented to ensure that the system remains robust in the face of network issues or other unexpected conditions. Logging statements are included to help with debugging and monitoring the server's and client's behavior.

## Future Enhancements

- **Destination Server Implementation**: Currently, the server simulates sending log messages to a destination server. A real implementation could be added to forward these messages to an actual logging server or storage system.
- **Configuration Options**: The server and client could be enhanced with command-line arguments or configuration files to make them more flexible.
- **Testing**: Unit tests and integration tests could be added to ensure the correctness and robustness of the implementation under various scenarios.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
