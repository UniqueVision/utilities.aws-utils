# Kinesis Data Streams

A Rust library providing utilities for AWS Kinesis Data Streams operations with built-in retry logic and batch processing capabilities.

## Features

- **Simple API**: Easy-to-use functions for putting records to Kinesis Data Streams
- **Batch Processing**: Efficient batch record operations with automatic size and count validation
- **Records Builder**: Builder pattern for constructing batches of records with size constraints
- **Error Handling**: Comprehensive error handling with custom error types
- **Retry Logic**: Built-in retry mechanisms for handling transient failures
- **AWS SDK Integration**: Built on top of the official AWS SDK for Rust
- **Testing Support**: Comprehensive unit tests with mocking capabilities

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kinesis_data_streams = "0.1.0"
```

## Usage

### Basic Usage

```rust
use kinesis_data_streams::{make_client_with_timeout_default, kinesis_data_stream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Kinesis client with default timeout settings
    let client = make_client_with_timeout_default(None).await;
    
    // Put a single record
    let result = kinesis_data_streams::add_record(
        &client,
        "my-stream",
        "partition-key",
        "Hello, Kinesis!".to_string(),
    ).await?;
    
    println!("Record added with sequence number: {}", result.sequence_number());
    
    Ok(())
}
```

### Batch Processing with RecordsBuilder

```rust
use kinesis_data_streams::{make_client_with_timeout_default, kinesis_data_stream, RecordsBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client_with_timeout_default(None).await;
    
    // Build a batch of records
    let mut builder = RecordsBuilder::new();
    builder.add_entry_data("Record 1".to_string())?;
    builder.add_entry_data("Record 2".to_string())?;
    builder.add_entry("Record 3".to_string(), Some("custom-partition".to_string()), None)?;
    
    // Send the batch
    let records = builder.build();
    let result = kinesis_data_streams::add_records(&client, "my-stream", records).await?;
    
    println!("Batch sent with {} failed records", result.failed_record_count().unwrap_or(0));
    
    Ok(())
}
```

### Custom Endpoint (for testing)

```rust
use kinesis_data_streams::make_client_with_timeout_default;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a custom endpoint (e.g., for LocalStack)
    let client = make_client_with_timeout_default(Some("http://localhost:4566".to_string())).await;
    
    // Your Kinesis operations here...
    
    Ok(())
}
```

### Timeout Configuration

```rust
use kinesis_data_streams::{make_client, make_client_with_timeout, make_client_with_timeout_default};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use default timeout settings (recommended)
    let client = make_client_with_timeout_default(None).await;
    
    // Use custom timeout settings
    let client = make_client_with_timeout(
        None, // endpoint_url
        Some(Duration::from_secs(3100)), // connect_timeout
        Some(Duration::from_secs(60)),   // operation_timeout
        Some(Duration::from_secs(55)),   // operation_attempt_timeout
        Some(Duration::from_secs(50)),   // read_timeout
    ).await;
    
    // Use legacy client without timeout configuration
    let client = make_client(None, None).await;
    
    Ok(())
}
```

## API Reference

### Functions

- `make_client_with_timeout_default(endpoint_url: Option<String>)` - Creates a Kinesis client with default timeout settings
- `make_client_with_timeout(endpoint_url, connect_timeout, operation_timeout, operation_attempt_timeout, read_timeout)` - Creates a Kinesis client with custom timeout settings
- `make_client(endpoint_url: Option<String>, timeout_config: Option<TimeoutConfig>)` - Creates a Kinesis client with optional custom endpoint and timeout configuration
- `kinesis_data_streams::add_record(client, stream_name, partition_key, data)` - Puts a single record
- `kinesis_data_streams::add_records(client, stream_name, records)` - Puts multiple records in batch

### RecordsBuilder

A builder for creating batches of records with automatic size validation:

- `new()` - Creates a new builder with default AWS limits
- `new_with_limit(single_limit, total_limit, record_limit)` - Creates a builder with custom limits
- `add_entry_data(data)` - Adds a record with auto-generated partition key
- `add_entry(data, partition_key, explicit_hash_key)` - Adds a record with custom keys
- `build()` - Builds the final vector of records
- `len()` - Returns the number of records
- `is_empty()` - Checks if the builder is empty

### Error Handling

The library provides comprehensive error handling through the `Error` enum using the `thiserror` crate:

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] Box<aws_sdk_kinesis::error::BuildError>),
    
    #[error("EntryOverAll {0}")]
    EntryOverAll(String),
    
    #[error("EntryOverItem {0}")]
    EntryOverItem(String),
    
    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_kinesis::Error>),
}
```

Error variants:
- `BuildError` - Errors when building AWS SDK request entries
- `EntryOverItem` - Individual record exceeds the 1MB size limit
- `EntryOverAll` - Adding a record would exceed batch limits (5MB total or 500 records)
- `AwsSdk` - General AWS SDK errors (network issues, authentication, etc.)

#### Error Handling Example

```rust
use kinesis_data_streams::{make_client_with_timeout_default, kinesis_data_stream, RecordsBuilder, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client_with_timeout_default(None).await;
    
    match kinesis_data_streams::add_record(&client, "my-stream", "key", "data").await {
        Ok(output) => println!("Success: {}", output.sequence_number()),
        Err(Error::AwsSdk(e)) => {
            // Handle AWS SDK errors (e.g., stream not found, throttling)
            eprintln!("AWS error: {}", e);
        }
        Err(e) => eprintln!("Other error: {}", e),
    }
    
    // Batch operations with size limit handling
    let mut builder = RecordsBuilder::new();
    match builder.add_entry_data("Large data...".to_string()) {
        Ok(()) => println!("Record added to batch"),
        Err(Error::EntryOverItem(msg)) => {
            // Single record too large
            eprintln!("Record too large: {}", msg);
        }
        Err(Error::EntryOverAll(msg)) => {
            // Batch is full, need to send current batch
            eprintln!("Batch full: {}", msg);
        }
        Err(e) => eprintln!("Unexpected error: {}", e),
    }
    
    Ok(())
}
```

## AWS Kinesis Limits

The library respects AWS Kinesis Data Streams limits:

- **Single Record**: Maximum 1MB per record
- **Batch Operation**: Maximum 5MB total payload and 500 records per batch
- **Partition Key**: Maximum 256 UTF-8 characters

These limits are enforced by the `RecordsBuilder` to prevent API errors.

## Testing

Run the test suite:

```bash
cargo test
```

For integration tests with specific environment variables:

```bash
RUST_LOG=info REALM_CODE=test cargo test test_kinesis_data_streams_records -- --nocapture --test-threads=1
```

The library includes comprehensive unit tests with mocking capabilities using `mockito` for testing without actual AWS resources.

## Configuration

### Authentication

The client uses the AWS SDK's default credential chain for authentication:

- Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`)
- ECS task role (for Fargate/ECS)
- EC2 instance profile
- AWS credentials file
- Other configured credential providers

## Dependencies

- `aws-config` - AWS configuration management
- `aws-sdk-kinesis` - Official AWS Kinesis SDK
- `thiserror` - Error handling
- `tracing` - Logging and tracing
- `uuid` - UUID generation for partition keys

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.