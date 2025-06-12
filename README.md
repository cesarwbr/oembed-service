# 🔗 oEmbed Service

A lightweight, fast oEmbed service written in Rust using Actix Web. This service provides oEmbed functionality for various URL providers including YouTube, Twitter/X, and generic websites through HTML meta tag parsing.

## Features

- **Multiple Provider Support**: Built-in support for YouTube and Twitter/X oEmbed endpoints
- **Fallback HTML Parsing**: Automatically extracts Open Graph and Twitter Card metadata for unsupported providers
- **Fast & Lightweight**: Built with Rust and Actix Web for high performance
- **JSON API**: Simple REST API endpoint that returns oEmbed-compliant JSON responses
- **Configurable Logging**: Comprehensive logging with configurable levels

## Prerequisites

- Rust 1.70+ and Cargo
- Internet connection (for fetching external URLs)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd oembed-service
```

2. Build the project:
```bash
cargo build --release
```

## Running the Service

### Basic Usage

To run the service with default settings:

```bash
cargo run
```

The service will start on `http://localhost:8080` by default.

### Running with Logging

The service uses `env_logger` for logging. You can control the log level using the `RUST_LOG` environment variable:

#### Different Log Levels

```bash
# Debug level (most verbose)
RUST_LOG=debug cargo run

# Info level (default, shows startup and request info)
RUST_LOG=info cargo run

# Warning level only
RUST_LOG=warn cargo run

# Error level only
RUST_LOG=error cargo run

# Trace level (extremely verbose)
RUST_LOG=trace cargo run
```

#### Module-Specific Logging

You can also enable logging for specific modules:

```bash
# Log only from the oembed-service crate
RUST_LOG=oembed_service=debug cargo run

# Log from multiple modules
RUST_LOG=oembed_service=debug,actix_web=info cargo run

# Log everything at debug level for your app, but only errors for dependencies
RUST_LOG=oembed_service=debug,actix_web=error cargo run
```

#### Production Logging

For production environments, consider using structured logging:

```bash
# Production run with info level
RUST_LOG=info cargo run --release

# Or with systemd/docker logging
RUST_LOG=info cargo run --release 2>&1 | logger -t oembed-service
```

## API Documentation

### Endpoint

**GET** `/oembed`

### Parameters

| Parameter | Type   | Required | Description                                    |
|-----------|--------|----------|------------------------------------------------|
| `url`     | string | Yes      | The URL to generate oEmbed data for           |

### Example Requests

```bash
# YouTube video
curl "http://localhost:8080/oembed?url=https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Twitter/X post
curl "http://localhost:8080/oembed?url=https://x.com/user/status/123456789"

# Generic website (fallback to HTML parsing)
curl "http://localhost:8080/oembed?url=https://example.com/article"
```

### Response Format

The service returns JSON responses compliant with the [oEmbed specification](https://oembed.com/):

```json
{
  "type": "rich",
  "version": "1.0",
  "title": "Example Title",
  "author_name": "Author Name",
  "author_url": "https://example.com/author",
  "provider_name": "Provider Name",
  "provider_url": "https://example.com",
  "thumbnail_url": "https://example.com/thumbnail.jpg",
  "thumbnail_width": 1200,
  "thumbnail_height": 630,
  "html": "<div>Embedded content HTML</div>",
  "width": 800,
  "height": 600
}
```

## Supported Providers

### Built-in oEmbed Support
- **YouTube**: `youtube.com/watch?v=*`, `youtu.be/*`
- **Twitter/X**: `x.com/*`, `twitter.com/*`

### Fallback HTML Parsing
For unsupported providers, the service automatically:
- Fetches the webpage HTML
- Extracts Open Graph metadata (`og:*`)
- Extracts Twitter Card metadata (`twitter:*`)
- Falls back to standard HTML meta tags and title

## Development

### Project Structure

```
src/
├── main.rs      # Application entry point and server setup
├── routes.rs    # API route handlers
├── provider.rs  # Core oEmbed provider logic
├── models.rs    # Data structures and serialization
└── errors.rs    # Error types and handling
```

### Adding New Providers

To add support for a new oEmbed provider, modify the `Provider::new()` method in `src/provider.rs`:

```rust
providers.insert(
    "newprovider.com".to_string(),
    ProviderConfig {
        oembed_endpoint: Some(Url::parse("https://newprovider.com/oembed").unwrap()),
        url_patterns: vec!["newprovider.com/".to_string()],
    },
);
```

### Running Tests

```bash
cargo test
```

### Development with Auto-Reload

For development with automatic reloading, install and use `cargo-watch`:

```bash
cargo install cargo-watch
RUST_LOG=debug cargo watch -x run
```

## Configuration

### Environment Variables

| Variable      | Description                | Default | Example Values |
|---------------|----------------------------|---------|----------------|
| `RUST_LOG`    | Logging level/filter       | `info`  | `debug`, `warn`, `error`, `trace` |

### Customizing Port and Host

To run on a different port or host, modify the `bind()` call in `src/main.rs`:

```rust
.bind("0.0.0.0:3000")?  // Listen on all interfaces, port 3000
```

## Docker Support

Create a `Dockerfile` for containerized deployment:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/oembed-service /usr/local/bin/oembed-service
EXPOSE 8080
ENV RUST_LOG=info
CMD ["oembed-service"]
```

Build and run:
```bash
docker build -t oembed-service .
docker run -p 8080:8080 -e RUST_LOG=debug oembed-service
```

## License

MIT

