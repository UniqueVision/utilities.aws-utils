# AWS S3 Utilities

A utility crate for AWS S3 client operations.

## Features

### Object Listing
- `list_stream` - Stream objects from an S3 bucket
- `list_all` - Retrieve all objects from an S3 bucket at once

### Object Operations
- `get_object` - Retrieve an object
- `get_object_string` - Retrieve object content as a string
- `get_object_buf_reader` - Get object as a BufferedReader
- `put_object` - Upload an object
- `put_object_from_path` - Upload an object from a file path
- `delete_object` - Delete a single object
- `delete_objects` - Batch delete objects matching a prefix

### Presigned URLs
- `put_presigned` - Generate a presigned URL for uploads
- `get_presigned` - Generate a presigned URL for downloads
- `presigned_url` - Extract URL string from PresignedRequest

## Usage Examples

```rust
use aws_sdk_s3::Client;
use aws_utils_s3::s3;

// List objects
let objects = s3::list_all(&client, "my-bucket", "prefix/").await?;

// Get object
let object = s3::get_object(&client, "my-bucket", "key.txt").await?;
let (content_type, content) = s3::get_object_string(object).await?;

// Upload object
s3::put_object(
    &client,
    "my-bucket",
    "key.txt",
    "Hello, World!",
    Some("text/plain"),
    None,
).await?;

// Upload from file
s3::put_object_from_path(
    &client,
    "my-bucket",
    "key.pdf",
    "/path/to/file.pdf",
    Some("application/pdf"),
    None,
).await?;

// Generate presigned URL
let presigned = s3::get_presigned(
    &client,
    "my-bucket",
    "key.txt",
    std::time::Duration::from_secs(3600),
).await?;
let url = s3::presigned_url(&presigned);

// Batch delete objects with prefix
s3::delete_objects(&client, "my-bucket", "temp/").await?;
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