# AWS S3 Utilities

A utility crate for AWS S3 client operations.

## Features

### Client Setup
- `make_client` - Create an S3 client with optional endpoint URL configuration

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
use aws_utils_s3::{bucket, object, presigned, make_client};

// Create client
let client = make_client(None).await;

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