# aws_utils_secretsmanager

AWS Secrets Manager utilities for retrieving secret values from AWS Secrets Manager.

## Features

- Simple interface for retrieving secrets from AWS Secrets Manager
- Support for secret versioning with version ID and version stage
- Custom error handling with detailed error types
- Support for custom AWS endpoints (useful for testing with LocalStack)
- Automatic fallback to dummy credentials for testing environments

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_secretsmanager = "0.1.0"
```

## Usage

### Basic Example

```rust
use aws_utils_secretsmanager::{make_client, secretsmanager::get_secret_value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Secrets Manager client
    let client = make_client(None).await;
    
    // Get secret value
    let secret = get_secret_value(&client, "my-secret-name").await?;
    println!("Secret value: {}", secret);
    
    Ok(())
}
```

### Using Custom Endpoint

```rust
use aws_utils_secretsmanager::{make_client, secretsmanager::get_secret_value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom endpoint (e.g., LocalStack)
    let client = make_client(Some("http://localhost:4566".to_string())).await;
    
    let secret = get_secret_value(&client, "test-secret").await?;
    println!("Secret value: {}", secret);
    
    Ok(())
}
```

### Getting Raw Secret Output with Versioning

```rust
use aws_utils_secretsmanager::{make_client, secretsmanager::get_secret_value_raw};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client(None).await;
    
    // Get specific version of secret
    let output = get_secret_value_raw(
        &client,
        Some("my-secret-name"),
        Some("version-uuid"),      // Version ID
        Some("AWSCURRENT")         // Version stage
    ).await?;
    
    if let Some(secret_string) = output.secret_string() {
        println!("Secret: {}", secret_string);
    }
    
    if let Some(version_id) = output.version_id() {
        println!("Version ID: {}", version_id);
    }
    
    Ok(())
}
```

### Getting Latest Secret Version

```rust
use aws_utils_secretsmanager::{make_client, secretsmanager::get_secret_value_raw};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client(None).await;
    
    // Get current version (default behavior)
    let output = get_secret_value_raw(
        &client,
        Some("my-secret-name"),
        None::<String>,           // No specific version ID
        Some("AWSCURRENT")        // Get current version
    ).await?;
    
    println!("Current secret: {:?}", output.secret_string());
    
    Ok(())
}
```

## API Reference

### Functions

#### `make_client(endpoint_url: Option<String>) -> Client`

Creates an AWS Secrets Manager client with optional custom endpoint URL.

- `endpoint_url`: Optional custom endpoint URL for testing (e.g., LocalStack)
- Returns: Configured AWS Secrets Manager Client

#### `get_secret_value(client: &Client, secret_id: &str) -> Result<String, Error>`

Retrieves a secret value as a string from the current version.

- `client`: AWS Secrets Manager client
- `secret_id`: Secret identifier (name or ARN)
- Returns: Secret value as String

#### `get_secret_value_raw(client: &Client, secret_id: Option<impl Into<String>>, version_id: Option<impl Into<String>>, version_stage: Option<impl Into<String>>) -> Result<GetSecretValueOutput, Error>`

Retrieves raw secret output from AWS Secrets Manager with version control.

- `client`: AWS Secrets Manager client
- `secret_id`: Optional secret identifier (name or ARN)
- `version_id`: Optional version UUID to retrieve specific version
- `version_stage`: Optional version stage (e.g., "AWSCURRENT", "AWSPENDING")
- Returns: Raw GetSecretValueOutput from AWS SDK

### Error Types

The crate defines custom error types:

- `Error::BuildError`: AWS SDK build errors
- `Error::AwsSdk`: AWS SDK service errors
- `Error::ValidationError`: Validation errors
- `Error::NotFound`: Secret not found

## Secret Versioning

AWS Secrets Manager supports versioning of secrets. You can:

- Get the current version using `"AWSCURRENT"` stage
- Get the pending version using `"AWSPENDING"` stage
- Get a specific version using the version UUID
- Let AWS choose the version by omitting version parameters

### Version Stages

- `AWSCURRENT`: The current version of the secret
- `AWSPENDING`: The version that will become current after rotation completes
- Custom stages: You can define custom version stages for your workflow

## Testing

Set up your test environment:

```bash
# Optional: Custom Secrets Manager endpoint (e.g., LocalStack)
export SECRETSMANAGER_ENDPOINT_URL=http://localhost:4566

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
cargo test test_get_secret_value -- --nocapture
```

## Environment Variables

The crate automatically sets dummy AWS credentials if they're not present:

- `AWS_ACCESS_KEY_ID`: Set to "dummy_access_key" if not present
- `AWS_SECRET_ACCESS_KEY`: Set to "dummy_secret_key" if not present  
- `AWS_REGION`: Set to "us-west-2" if not present

This makes it easy to use in testing environments without requiring real AWS credentials.

## Use Cases

### Database Credentials

```rust
use aws_utils_secretsmanager::{make_client, secretsmanager::get_secret_value};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client(None).await;
    let secret_json = get_secret_value(&client, "prod/db/credentials").await?;
    
    let credentials: Value = serde_json::from_str(&secret_json)?;
    let username = credentials["username"].as_str().unwrap();
    let password = credentials["password"].as_str().unwrap();
    
    // Use credentials to connect to database
    println!("Connecting as user: {}", username);
    
    Ok(())
}
```

### API Keys

```rust
use aws_utils_secretsmanager::{make_client, secretsmanager::get_secret_value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client(None).await;
    let api_key = get_secret_value(&client, "prod/external-api/key").await?;
    
    // Use API key for external service calls
    println!("API Key retrieved successfully");
    
    Ok(())
}
```

## License

MIT