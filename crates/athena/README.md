# aws_utils_athena

A Rust client library for AWS Athena, providing convenient utilities for query execution and result streaming.

## Features

- Simple client creation with configurable timeouts
- Query execution with support for all Athena parameters
- Asynchronous query execution with wait functionality
- Stream-based result retrieval for large datasets
- Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_athena = "0.4.0"
```

## Usage

### Creating a Client

```rust
use aws_utils_athena;

// Create client with default timeouts
let client = aws_utils_athena::make_client_with_timeout_default(None).await;

// Create client with custom timeouts
use std::time::Duration;
let client = aws_utils_athena::make_client_with_timeout(
    None, // endpoint_url
    Some(Duration::from_secs(50)), // connect_timeout
    Some(Duration::from_secs(60)), // operation_timeout
    Some(Duration::from_secs(55)), // operation_attempt_timeout
    Some(Duration::from_secs(45)), // read_timeout
).await;

// Create client without timeout configuration (legacy)
let client = aws_utils_athena::make_client(None, None, None).await;

// Create client with custom endpoint and no timeout (legacy)
let client = aws_utils_athena::make_client(
    Some("http://localhost:9000".to_string()),
    None,
    None,
).await;
```

### Logging AWS Communication

`make_client` accepts an optional [`SharedInterceptor`]. By passing an interceptor that
implements `aws_sdk_athena::config::Intercept`, you can run custom logic — such as
logging — every time the client communicates with AWS.

The interceptor below logs each request, response, and operation result. It uses the
[`tracing`](https://crates.io/crates/tracing) crate, which is also what the AWS SDK uses
internally.

```rust
use aws_utils_athena::make_client;
use aws_sdk_athena::config::{
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
        "AthenaLoggingInterceptor"
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
            "Athena -> AWS request"
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
        tracing::info!(status = %response.status(), "AWS -> Athena response");
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
            Ok(_) => tracing::info!("Athena operation succeeded"),
            Err(err) => tracing::warn!(error = %err, "Athena operation failed"),
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
INFO AthenaLoggingInterceptor: Athena -> AWS request method=POST uri=https://athena.ap-northeast-1.amazonaws.com/
INFO AthenaLoggingInterceptor: AWS -> Athena response status=200
INFO AthenaLoggingInterceptor: Athena operation succeeded
```

### Executing Queries

```rust
use aws_utils_athena::query;
use aws_sdk_athena::types::{QueryExecutionContext, ResultConfiguration};

// Start query execution
let result = query::start_query_execution(
    &client,
    Some("SELECT * FROM my_table LIMIT 10"),
    Some(QueryExecutionContext::builder()
        .database("my_database")
        .build()),
    Some(ResultConfiguration::builder()
        .output_location("s3://my-bucket/results/")
        .build()),
    None, // client_request_token
    None, // execution_parameters
    None, // result_reuse_configuration
    None, // work_group
).await?;

// Get query execution status
let execution_id = result.query_execution_id();
let status = query::get_query_execution(&client, execution_id).await?;
```

### Waiting for Query Completion

```rust
use aws_utils_athena::wait;
use std::time::Duration;

// Execute query and wait for completion
let builder = client.start_query_execution()
    .query_string("SELECT * FROM my_table")
    .query_execution_context(
        QueryExecutionContext::builder()
            .database("my_database")
            .build()
    );

let query_execution_id = wait::start_query_execution_wait(
    &client,
    builder,
    Duration::from_secs(300), // timeout
    Duration::from_secs(2),   // check interval
).await?;
```

### Streaming Query Results

```rust
use aws_utils_athena::query;
use futures_util::TryStreamExt;

// Get results as a stream
let stream = query::get_query_results_stream(&client, Some(query_execution_id));

// Process results
futures_util::pin_mut!(stream);
while let Some(result_set) = stream.try_next().await? {
    // Process each ResultSet
    if let Some(rows) = result_set.rows() {
        for row in rows {
            // Process row data
        }
    }
}
```

## Error Handling

The library provides a comprehensive `Error` enum for handling various failure cases:

```rust
use aws_utils_athena::error::Error;

match query::start_query_execution(&client, query_string, None, None, None, None, None, None).await {
    Ok(output) => {
        // Handle success
    }
    Err(Error::Invalid(msg)) => {
        // Handle invalid input
    }
    Err(Error::QueryFailed(query_execution)) => {
        // Handle query failure
        // query_execution contains the failed QueryExecution details
    }
    Err(Error::QueryCancelled) => {
        // Handle query cancellation
    }
    Err(Error::Timeout(_)) => {
        // Handle timeout
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