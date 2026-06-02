# aws_utils_ssm

AWS SSM utilities for getting parameter values from AWS Systems Manager Parameter Store.

## Features

- Simple interface for retrieving SSM parameters
- Support for encrypted parameters with automatic decryption
- Custom error handling with detailed error types
- Support for custom AWS endpoints (useful for testing with LocalStack)
- Support for AWS SDK's default credential chain

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_ssm = "0.1.0"
```

## Usage

### Basic Example

```rust
use aws_utils_ssm::{make_client_with_timeout_default, ssm::get_parameter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create SSM client with default timeout configuration
    let client = make_client_with_timeout_default(None).await;
    
    // Get parameter value
    let value = get_parameter(&client, "/my/parameter/name").await?;
    println!("Parameter value: {}", value);
    
    Ok(())
}
```

### Using Custom Endpoint

```rust
use aws_utils_ssm::{make_client_with_timeout_default, ssm::get_parameter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom endpoint (e.g., LocalStack)
    let client = make_client_with_timeout_default(Some("http://localhost:4566".to_string())).await;
    
    let value = get_parameter(&client, "/test/parameter").await?;
    println!("Parameter value: {}", value);
    
    Ok(())
}
```

### Getting Raw Parameter Output

```rust
use aws_utils_ssm::{make_client_with_timeout_default, ssm::get_parameter_raw};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = make_client_with_timeout_default(None).await;
    
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

### Using Custom Timeout Configuration

```rust
use std::time::Duration;
use aws_utils_ssm::{make_client_with_timeout, ssm::get_parameter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with custom timeout settings
    let client = make_client_with_timeout(
        None,
        Some(Duration::from_secs(5)),      // 5 second connect timeout
        Some(Duration::from_secs(30)),     // 30 second operation timeout
        Some(Duration::from_secs(25)),     // 25 second operation attempt timeout
        Some(Duration::from_secs(20)),     // 20 second read timeout
    ).await;
    
    let value = get_parameter(&client, "/my/parameter").await?;
    println!("Parameter value: {}", value);
    
    Ok(())
}
```

### Using with TimeoutConfig

```rust
use aws_config::timeout::{TimeoutConfig, TimeoutConfigBuilder};
use aws_utils_ssm::{make_client, ssm::get_parameter};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build custom timeout configuration
    let timeout_config = TimeoutConfigBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .operation_timeout(Duration::from_secs(120))
        .build();
    
    // Create client with custom timeout configuration
    let client = make_client(None, Some(timeout_config), None).await;
    
    let value = get_parameter(&client, "/my/parameter").await?;
    println!("Parameter value: {}", value);
    
    Ok(())
}
```

### Logging AWS Communication

`make_client` accepts an optional [`SharedInterceptor`]. By passing an interceptor that
implements `aws_sdk_ssm::config::Intercept`, you can run custom logic — such as logging —
every time the client communicates with AWS.

The interceptor below logs each request, response, and operation result. It uses the
[`tracing`](https://crates.io/crates/tracing) crate, which is also what the AWS SDK uses
internally.

```rust
use aws_utils_ssm::make_client;
use aws_sdk_ssm::config::{
    ConfigBag, Intercept, RuntimeComponents, SharedInterceptor,
    interceptors::{
        AfterDeserializationInterceptorContextRef, BeforeDeserializationInterceptorContextRef,
        BeforeTransmitInterceptorContextRef,
    },
};

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
struct LoggingInterceptor;

impl Intercept for LoggingInterceptor {
    fn name(&self) -> &'static str {
        "SsmLoggingInterceptor"
    }

    // Called just before each HTTP request is sent (once per retry attempt).
    fn read_before_transmit(
        &self,
        context: &BeforeTransmitInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let request = context.request();
        tracing::info!(
            method = %request.method(),
            uri = %request.uri(),
            "SSM -> AWS request"
        );
        Ok(())
    }

    // Called right after each HTTP response is received.
    fn read_before_deserialization(
        &self,
        context: &BeforeDeserializationInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let response = context.response();
        tracing::info!(status = %response.status(), "AWS -> SSM response");
        Ok(())
    }

    // Called once when the operation completes (after retries), with success or error.
    fn read_after_deserialization(
        &self,
        context: &AfterDeserializationInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        match context.output_or_error() {
            Ok(_) => tracing::info!("SSM operation succeeded"),
            Err(err) => tracing::warn!(error = %err, "SSM operation failed"),
        }
        Ok(())
    }
}

# async fn run() {
// Pass the interceptor as the third argument.
let client = make_client(None, None, Some(SharedInterceptor::new(LoggingInterceptor))).await;
# }
```

`tracing` does not emit anything until a subscriber is initialized. Set one up once in your
application (for example with `tracing-subscriber`) and control verbosity with `RUST_LOG`:

```rust
// Add `tracing-subscriber` to your dependencies.
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()),
    )
    .init();
```

Example output (`RUST_LOG=info`):

```text
INFO SsmLoggingInterceptor: SSM -> AWS request method=POST uri=https://ssm.ap-northeast-1.amazonaws.com/
INFO SsmLoggingInterceptor: AWS -> SSM response status=200
INFO SsmLoggingInterceptor: SSM operation succeeded
```

## API Reference

### Functions

#### `make_client_with_timeout_default(endpoint_url: Option<String>) -> Client`

Creates an AWS SSM client with default timeout configuration.

- `endpoint_url`: Optional custom endpoint URL for testing (e.g., LocalStack)
- Returns: Configured AWS SSM Client with default timeouts
- Default timeouts:
  - Connect timeout: 3100 seconds
  - Operation timeout: 60 seconds
  - Operation attempt timeout: 55 seconds
  - Read timeout: 50 seconds

#### `make_client_with_timeout(endpoint_url: Option<String>, connect_timeout: Option<Duration>, operation_timeout: Option<Duration>, operation_attempt_timeout: Option<Duration>, read_timeout: Option<Duration>) -> Client`

Creates an AWS SSM client with custom timeout configuration.

- `endpoint_url`: Optional custom endpoint URL for testing (e.g., LocalStack)
- `connect_timeout`: Optional timeout for establishing connections
- `operation_timeout`: Optional timeout for entire operations
- `operation_attempt_timeout`: Optional timeout for individual operation attempts
- `read_timeout`: Optional timeout for reading responses
- Returns: Configured AWS SSM Client with custom timeouts

#### `make_client(endpoint_url: Option<String>, timeout_config: Option<TimeoutConfig>, interceptor: Option<SharedInterceptor>) -> Client`

Creates an AWS SSM client with optional custom endpoint URL, timeout configuration, and interceptor.

- `endpoint_url`: Optional custom endpoint URL for testing (e.g., LocalStack)
- `timeout_config`: Optional timeout configuration
- `interceptor`: Optional interceptor for running custom logic (e.g. logging) on every AWS communication
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

## Authentication

The client uses the AWS SDK's default credential chain for authentication:

- Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`)
- ECS task role (for Fargate/ECS)
- EC2 instance profile
- AWS credentials file
- Other configured credential providers

## License

MIT