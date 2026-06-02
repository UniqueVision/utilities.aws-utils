# AWS S3 Utilities

A utility crate for AWS S3 client operations.

## Features

### Client Setup
- `make_client_with_timeout_default` - Create an S3 client with default timeout settings
- `make_client_with_timeout` - Create an S3 client with custom timeout settings
- `make_client` - Create an S3 client with optional endpoint URL, timeout configuration, and interceptor (e.g. for logging)

### Bucket Operations
- `bucket::create_bucket` - Create a new S3 bucket
- `bucket::list_stream` - Stream buckets with prefix filtering
- `bucket::list_all` - List all buckets matching a prefix
- `bucket::delete_bucket` - Delete a bucket and all its contents
- `bucket::delete_buckets` - Delete multiple buckets matching a prefix

### Object Operations
- `object::list_stream` - Stream objects from an S3 bucket with optional prefix
- `object::list_all` - Retrieve all objects from an S3 bucket at once
- `object::get_object` - Retrieve an object
- `object::is_exists` - Check if an object exists
- `object::get_object_string` - Retrieve object content as a string
- `object::get_object_buf_reader` - Get object as a BufferedReader
- `object::put_object` - Upload an object
- `object::put_object_from_path` - Upload an object from a file path
- `object::delete_object` - Delete a single object
- `object::delete_objects` - Batch delete objects matching a prefix
- `object::copy_object` - Copy an object between buckets
- `object::copy_objects_prefix` - Copy multiple objects matching a prefix

### Presigned URLs
- `presigned::put_presigned` - Generate a presigned URL for uploads
- `presigned::get_presigned` - Generate a presigned URL for downloads
- `presigned::presigned_url` - Extract URL string from PresignedRequest

## Usage Examples

```rust
use aws_utils_s3::{bucket, object, presigned, make_client_with_timeout_default};

// Create client with default timeout settings
let client = make_client_with_timeout_default(None).await;

// Bucket operations
bucket::create_bucket(&client, "my-bucket").await?;
let buckets = bucket::list_all(&client, "my-").await?;
bucket::delete_bucket(&client, "old-bucket").await?;

// List objects
let objects = object::list_all(&client, "my-bucket", Some("prefix/")).await?;

// Check if object exists
let exists = object::is_exists(&client, "my-bucket", "key.txt").await?;

// Get object
let object = object::get_object(&client, "my-bucket", "key.txt").await?;
let (content_type, content) = object::get_object_string(object).await?;

// Upload object
object::put_object(
    &client,
    "my-bucket",
    "key.txt",
    "Hello, World!",
    Some("text/plain"),
    None,
).await?;

// Upload from file
object::put_object_from_path(
    &client,
    "my-bucket",
    "key.pdf",
    "/path/to/file.pdf",
    Some("application/pdf"),
    None,
).await?;

// Copy object
object::copy_object(
    &client,
    "src-bucket",
    "src-key.txt",
    "dst-bucket",
    "dst-key.txt",
).await?;

// Copy objects with prefix
object::copy_objects_prefix(
    &client,
    "src-bucket",
    "src-prefix",
    "dst-bucket",
    "dst-prefix",
).await?;

// Generate presigned URL
let presigned = presigned::get_presigned(
    &client,
    "my-bucket",
    "key.txt",
    std::time::Duration::from_secs(3600),
).await?;
let url = presigned::presigned_url(&presigned);

// Batch delete objects with prefix
object::delete_objects(&client, "my-bucket", Some("temp/")).await?;
```

## Timeout Configuration

```rust
use aws_utils_s3::{make_client, make_client_with_timeout, make_client_with_timeout_default};
use std::time::Duration;

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

// Use custom endpoint with default timeout settings
let client = make_client_with_timeout_default(
    Some("http://localhost:4566".to_string())
).await;

// Use legacy client without timeout configuration
let client = make_client(None, None, None).await;

// Use custom endpoint and no timeout (legacy)
let client = make_client(Some("http://localhost:4566".to_string()), None, None).await;
```

## Logging AWS Communication

`make_client` (and `make_client_with_credentials`) accepts an optional [`SharedInterceptor`].
By passing an interceptor that implements `aws_sdk_s3::config::Intercept`, you can run custom
logic — such as logging — every time the client communicates with AWS.

The interceptor below logs each request, response, and operation result. It uses the
[`tracing`](https://crates.io/crates/tracing) crate, which is also what the AWS SDK uses
internally.

```rust
use aws_utils_s3::make_client;
use aws_sdk_s3::config::{
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
        "S3LoggingInterceptor"
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
            "S3 -> AWS request"
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
        tracing::info!(status = %response.status(), "AWS -> S3 response");
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
            Ok(_) => tracing::info!("S3 operation succeeded"),
            Err(err) => tracing::warn!(error = %err, "S3 operation failed"),
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
INFO S3LoggingInterceptor: S3 -> AWS request method=GET uri=https://my-bucket.s3.ap-northeast-1.amazonaws.com/key.txt
INFO S3LoggingInterceptor: AWS -> S3 response status=200
INFO S3LoggingInterceptor: S3 operation succeeded
```

## Error Handling

This crate provides an `Error` type that handles:
- AWS SDK errors
- Build errors
- ByteStream errors
- Presigning configuration errors
- I/O errors
- Validation errors

Helper methods for specific error checking:
- `is_no_such_key()` - Check if object doesn't exist
- `is_no_such_bucket()` - Check if bucket doesn't exist

## Notes

- `delete_objects` processes in batches of 1000 (due to AWS S3 limitations)
- Stream processing enables efficient handling of large numbers of objects