# firehose

AWS Kinesis Data Firehose utilities for Rust.

## Overview

This crate provides a simple and efficient client for interacting with AWS Kinesis Data Firehose. It includes:

- Client creation with configurable timeout settings
- Error handling using `thiserror`
- Simple API for putting records to Firehose delivery streams

## Features

- **Flexible Client Configuration**: Create clients with default or custom timeout settings
- **Endpoint Configuration**: Support for custom endpoints (useful for local development with LocalStack)
- **Type-safe Error Handling**: Comprehensive error types using `thiserror`
- **Async/Await Support**: Fully async API built on top of `aws-sdk-firehose`

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
firehose = { path = "../firehose" }
```

## Usage

### Creating a Client

#### Default Client with Standard Timeouts

```rust
use firehose::make_client_with_timeout_default;

#[tokio::main]
async fn main() {
    let client = make_client_with_timeout_default(None).await;
}
```

#### Client with Custom Timeouts

```rust
use firehose::make_client_with_timeout;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = make_client_with_timeout(
        None, // endpoint_url
        Some(Duration::from_secs(30)),  // connect_timeout
        Some(Duration::from_secs(60)),  // operation_timeout
        Some(Duration::from_secs(55)),  // operation_attempt_timeout
        Some(Duration::from_secs(50)),  // read_timeout
    ).await;
}
```

#### Client with Custom Endpoint (LocalStack)

```rust
use firehose::make_client_with_timeout_default;

#[tokio::main]
async fn main() {
    let endpoint_url = Some("http://localhost:4566".to_string());
    let client = make_client_with_timeout_default(endpoint_url).await;
}
```

### Putting Records to Firehose

```rust
use firehose::{make_client_with_timeout_default, firehose::put_record};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client_with_timeout_default(None).await;
    
    let data = serde_json::json!({
        "user_id": "123",
        "event": "page_view",
        "timestamp": "2024-01-01T00:00:00Z"
    });
    
    let result = put_record(
        &client,
        "my-delivery-stream",
        data.to_string().as_bytes()
    ).await?;
    
    println!("Record ID: {:?}", result.record_id());
    Ok(())
}
```

## Environment Variables

The client automatically sets default AWS credentials and region if not already configured:

- `AWS_ACCESS_KEY_ID`: Defaults to "dummy_access_key" if not set
- `AWS_SECRET_ACCESS_KEY`: Defaults to "dummy_secret_key" if not set
- `AWS_REGION`: Defaults to "us-west-2" if not set

## Error Handling

The crate provides a comprehensive `Error` enum that wraps AWS SDK errors:

```rust
use firehose::error::Error;

match put_record(&client, "stream", data).await {
    Ok(output) => println!("Success: {:?}", output.record_id()),
    Err(Error::AwsSdk(e)) => eprintln!("AWS error: {}", e),
    Err(Error::Invalid(msg)) => eprintln!("Invalid input: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## License

See the parent project's LICENSE file for details.