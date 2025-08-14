# aws_utils_lambda

AWS Lambda utilities for Rust, providing a simplified interface for AWS Lambda operations.

## Features

- Easy client creation with automatic credential handling
- Lambda function invocation with comprehensive parameter support
- Error handling with custom error types
- Re-exports `aws_sdk_lambda` for direct access to AWS SDK types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_lambda = "0.1.0"
```

## Usage

### Creating a Lambda Client

```rust
use aws_utils_lambda;

#[tokio::main]
async fn main() {
    // Create client with default timeout configuration
    let client = aws_utils_lambda::make_client_with_timeout_default(None).await;
    
    // Create client with custom endpoint (e.g., for LocalStack)
    let client = aws_utils_lambda::make_client_with_timeout_default(Some("http://localhost:4566".to_string())).await;
}
```

The client creation functions automatically handle AWS credentials:
- Uses existing AWS environment variables if set
- Sets dummy values for local development if not configured
- Defaults to `us-west-2` region if `AWS_REGION` is not set

### Timeout Configuration

```rust
use aws_utils_lambda::{make_client, make_client_with_timeout, make_client_with_timeout_default};
use std::time::Duration;

#[tokio::main]
async fn main() {
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
}
```

### Invoking Lambda Functions

```rust
use aws_utils_lambda::{lambda, aws_sdk_lambda::types::{InvocationType, LogType}};
use aws_sdk_lambda::primitives::Blob;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = aws_utils_lambda::make_client_with_timeout_default(None).await;
    
    // Simple invocation
    let result = lambda::invoke(
        &client,
        Some("my-function"),
        None,  // client_context
        None,  // invocation_type (defaults to RequestResponse)
        None,  // log_type
        Some(Blob::new(r#"{"key": "value"}"#)),
        None,  // qualifier
    ).await?;
    
    // Async invocation
    let result = lambda::invoke(
        &client,
        Some("my-function"),
        None,
        Some(InvocationType::Event),
        Some(LogType::Tail),
        Some(Blob::new(r#"{"key": "value"}"#)),
        Some("$LATEST"),
    ).await?;
    
    Ok(())
}
```

### Error Handling

The crate provides custom error types that wrap AWS SDK errors:

```rust
use aws_utils_lambda::error::Error;

match lambda::invoke(&client, Some("my-function"), None, None, None, None, None).await {
    Ok(output) => {
        // Handle successful response
    }
    Err(Error::AwsSdk(e)) => {
        // Handle AWS SDK errors
    }
    Err(Error::BuildError(e)) => {
        // Handle build errors
    }
    Err(Error::ValidationError(msg)) => {
        // Handle validation errors
    }
}
```

## API Reference

### Client Creation Functions

- `make_client_with_timeout_default(endpoint_url: Option<String>)` - Creates a Lambda client with default timeout settings
- `make_client_with_timeout(endpoint_url, connect_timeout, operation_timeout, operation_attempt_timeout, read_timeout)` - Creates a Lambda client with custom timeout settings
- `make_client(endpoint_url: Option<String>, timeout_config: Option<TimeoutConfig>)` - Creates a Lambda client with optional custom endpoint and timeout configuration

### Lambda Functions

- `lambda::invoke(client, function_name, client_context, invocation_type, log_type, payload, qualifier)` - Invokes a Lambda function with comprehensive parameter support

## Re-exports

The crate re-exports `aws_sdk_lambda` for direct access to AWS SDK types:

```rust
use aws_utils_lambda::aws_sdk_lambda::{types::InvocationType, primitives::Blob};
```

## License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.