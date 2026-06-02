# aws_utils_firehose

AWS Kinesis Data Firehose utilities for Rust, providing a simplified interface for sending records to delivery streams.

## Features

- Simple Firehose client creation with configurable endpoint and timeouts
- Sending records to a delivery stream
- Optional interceptor support for logging AWS communication
- Error handling with custom error types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_firehose = "0.3.0"
```

## Usage

### Creating a Client

```rust
use aws_utils_firehose;
use std::time::Duration;

// Create client with default timeout settings
let client = aws_utils_firehose::make_client_with_timeout_default(None).await;

// Create client with custom timeout settings
let client = aws_utils_firehose::make_client_with_timeout(
    None, // endpoint_url
    Some(Duration::from_secs(3100)), // connect_timeout
    Some(Duration::from_secs(60)),   // operation_timeout
    Some(Duration::from_secs(55)),   // operation_attempt_timeout
    Some(Duration::from_secs(50)),   // read_timeout
).await;

// Create client without timeout configuration (legacy)
let client = aws_utils_firehose::make_client(None, None, None).await;

// Create client with custom endpoint and no timeout (legacy)
let client = aws_utils_firehose::make_client(
    Some("http://localhost:4566".to_string()),
    None,
    None,
).await;
```

### Logging AWS Communication

`make_client` accepts an optional [`SharedInterceptor`]. By passing an interceptor that
implements `aws_sdk_firehose::config::Intercept`, you can run custom logic — such as
logging — every time the client communicates with AWS.

The interceptor below logs each request, response, and operation result. It uses the
[`tracing`](https://crates.io/crates/tracing) crate, which is also what the AWS SDK uses
internally.

```rust
use aws_utils_firehose::make_client;
use aws_sdk_firehose::config::{
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
        "FirehoseLoggingInterceptor"
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
            "Firehose -> AWS request"
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
        tracing::info!(status = %response.status(), "AWS -> Firehose response");
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
            Ok(_) => tracing::info!("Firehose operation succeeded"),
            Err(err) => tracing::warn!(error = %err, "Firehose operation failed"),
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
INFO FirehoseLoggingInterceptor: Firehose -> AWS request method=POST uri=https://firehose.ap-northeast-1.amazonaws.com/
INFO FirehoseLoggingInterceptor: AWS -> Firehose response status=200
INFO FirehoseLoggingInterceptor: Firehose operation succeeded
```

### Sending Records

```rust
use aws_utils_firehose::firehose::put_record;

// Send a single record to a delivery stream
let output = put_record(
    &client,
    "my-delivery-stream",
    b"{\"event\":\"example\"}".to_vec(),
).await?;

println!("record_id = {}", output.record_id());
```

## Error Handling

The crate provides a custom `Error` type that wraps AWS SDK errors:

- `BuildError` - Failed to build a request input (e.g. an invalid `Record`)
- `AwsSdk` - AWS SDK specific errors
- `Invalid` - Invalid input or state

```rust
use aws_utils_firehose::error::Error;

match put_record(&client, "my-delivery-stream", b"data".to_vec()).await {
    Ok(output) => {
        // Handle success
    }
    Err(Error::Invalid(msg)) => {
        // Handle invalid input
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Authentication

The client uses the AWS SDK's default credential chain for authentication:
- Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`)
- ECS task role (for Fargate/ECS)
- EC2 instance profile
- AWS credentials file
- Other configured credential providers

## License

This project is part of the utilities.aws-utils workspace.
