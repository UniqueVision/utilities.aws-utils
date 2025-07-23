# aws_utils_ssm

AWS SSM utilities for getting parameter values from AWS Systems Manager Parameter Store.

## Features

- Simple interface for retrieving SSM parameters
- Support for encrypted parameters with automatic decryption
- Custom error handling with detailed error types
- Support for custom AWS endpoints (useful for testing with LocalStack)
- Automatic fallback to dummy credentials for testing environments

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_ssm = "0.1.0"
```

## Usage

### Basic Example

```rust
use aws_utils_ssm::{make_client, ssm::get_parameter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create SSM client
    let client = make_client(None).await;
    
    // Get parameter value
    let value = get_parameter(&client, "/my/parameter/name").await?;
    println!("Parameter value: {}", value);
    
    Ok(())
}
```

### Using Custom Endpoint

```rust
use aws_utils_ssm::{make_client, ssm::get_parameter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom endpoint (e.g., LocalStack)
    let client = make_client(Some("http://localhost:4566".to_string())).await;
    
    let value = get_parameter(&client, "/test/parameter").await?;
    println!("Parameter value: {}", value);
    
    Ok(())
}
```

### Getting Raw Parameter Output

```rust
use aws_utils_ssm::{make_client, ssm::get_parameter_raw};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client(None).await;
    
    // Get full parameter information
    let output = get_parameter_raw(&client, Some("/my/parameter"), Some(true)).await?;
    
    if let Some(param) = output.parameter() {
        println!("Name: {:?}", param.name());
        println!("Type: {:?}", param.r#type());
        println!("Value: {:?}", param.value());
    }
    
    Ok(())
}
```

## API Reference

### Functions

#### `make_client(endpoint_url: Option<String>) -> Client`

Creates an AWS SSM client with optional custom endpoint URL.

- `endpoint_url`: Optional custom endpoint URL for testing (e.g., LocalStack)
- Returns: Configured AWS SSM Client

#### `get_parameter(client: &Client, name: &str) -> Result<String, Error>`

Retrieves a parameter value as a string with automatic decryption.

- `client`: AWS SSM client
- `name`: Parameter name (e.g., "/my/parameter/name")
- Returns: Parameter value as String

#### `get_parameter_raw(client: &Client, name: Option<impl Into<String>>, with_decryption: Option<bool>) -> Result<GetParameterOutput, Error>`

Retrieves raw parameter output from AWS SSM.

- `client`: AWS SSM client
- `name`: Optional parameter name
- `with_decryption`: Whether to decrypt the parameter value
- Returns: Raw GetParameterOutput from AWS SDK

### Error Types

The crate defines custom error types:

- `Error::BuildError`: AWS SDK build errors
- `Error::AwsSdk`: AWS SDK service errors
- `Error::ValidationError`: Validation errors
- `Error::NotFound`: Parameter not found

## Testing

The crate includes tests that require specific environment variables:

```bash
# Required for tests to run
export REALM_CODE=test

# Optional: Custom SSM endpoint (e.g., LocalStack)
export SSM_ENDPOINT_URL=http://localhost:4566

# Optional: Test parameter name (defaults to "/test/parameter")
export TEST_SSM_PARAMETER_NAME=/my/test/parameter

# Run tests
cargo test
```

### Test Commands

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=info cargo test -- --nocapture

# Run specific test
cargo test test_get_parameter -- --nocapture
```

## Environment Variables

The crate automatically sets dummy AWS credentials if they're not present:

- `AWS_ACCESS_KEY_ID`: Set to "dummy_access_key" if not present
- `AWS_SECRET_ACCESS_KEY`: Set to "dummy_secret_key" if not present  
- `AWS_REGION`: Set to "us-west-2" if not present

This makes it easy to use in testing environments without requiring real AWS credentials.

## License

MIT