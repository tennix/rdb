# RDB - Redis-compatible Server in Rust

A lightweight Redis-compatible server implementation written in Rust using async/await with Tokio.

## Features

- In-memory key-value store
- Support for basic Redis commands (SET, GET)
- RESP (Redis Serialization Protocol) protocol support
- Asynchronous I/O using Tokio
- Concurrent client handling
- Basic INFO and COMMAND support

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo (Rust's package manager)

### Installation

```bash
git clone https://github.com/tennix/rdb.git
cd rdb
cargo build --release
```

### Running the Server

```bash
cargo run
```

The server will start listening on `127.0.0.1:6379` (default Redis port).

## Usage

You can connect to the server using any Redis client. For example, using `redis-cli`:

```bash
redis-cli
```

### Supported Commands

- `SET key value` - Store a key-value pair
- `GET key` - Retrieve the value for a given key
- `INFO` - Get server information
- `COMMAND` - Get command information (minimal implementation)

### Example

```
> INFO
> SET mykey "Hello World"
OK
> GET mykey
"Hello World"
> SAVE
```

## Testing

Run the test suite with:

```bash
cargo test
```

## License

This project is open source and available under the MIT License.

## Contributing

The code is mostly written by AI. I want to investigate how far can AI go. So please DO NOT contribute to this repo at the moment.