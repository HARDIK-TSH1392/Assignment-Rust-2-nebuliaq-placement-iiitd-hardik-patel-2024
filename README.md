
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
- [Quickwit](https://quickwit.io/) (Log indexing tool)

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

## Bonus Task: Quickwit Integration

### Approach

For the bonus task, Quickwit was used as the indexing tool instead of Elasticsearch due to setup issues. Quickwit is a fast and lightweight search engine designed specifically for logs and metrics. The integration follows these steps:

- **Buffering and Flushing**: The server buffers log messages in memory until either 100 messages are accumulated or 10 seconds have passed. The buffered messages are then flushed and prepared for indexing.
  
- **Log Message Formatting**: The log messages are formatted as JSON documents, with each message including a timestamp. The JSON documents are then written to a temporary file for indexing.

- **Quickwit Indexing**: The server runs the Quickwit `index ingest` command to index the log messages. The command reads the JSON documents from the temporary file and indexes them into Quickwit. If the indexing operation fails, the error is logged for debugging purposes.

### Running the Server with Quickwit Integration

To start the server with Quickwit integration, make sure Quickwit is installed and configured properly. You can specify the path to your Quickwit binary when initializing the server in `main.rs`. Here’s an example of how to run the server:

```bash
cargo run --bin rust_log_indexer
```

The server will index the log messages into Quickwit when the buffer is flushed. The indexed logs can then be searched or queried using Quickwit’s search capabilities.

### Future Enhancements

- **Retry Mechanism**: Currently, the server logs errors if the Quickwit indexing fails. A retry mechanism could be added to attempt indexing multiple times before giving up.
- **Configuration**: Paths and other settings could be made configurable via environment variables or command-line arguments.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
