# aws_utils_sqs

A Rust library providing utilities for working with AWS Simple Queue Service (SQS).

## Features

- Queue management (create, delete)
- Message operations (send, receive, delete)
- Batch operations for sending and deleting messages
- Builder patterns for complex operations
- Type-safe queue attribute configuration
- FIFO queue support
- Dead letter queue configuration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_sqs = "0.1.0"
```

## Client Creation Functions

The library provides three functions for creating SQS clients with different timeout configurations:

### `make_client_with_timeout_default(endpoint_url: Option<String>) -> Client`

Creates an SQS client with default timeout settings optimized for typical SQS operations.

**Default timeout values:**
- Connect timeout: 3100 seconds
- Operation timeout: 60 seconds
- Operation attempt timeout: 55 seconds
- Read timeout: 50 seconds

### `make_client_with_timeout(...) -> Client`

Creates an SQS client with custom timeout settings. Accepts:
- `endpoint_url`: Optional custom endpoint URL
- `connect_timeout`: Optional timeout for establishing connections
- `operation_timeout`: Optional timeout for entire operations
- `operation_attempt_timeout`: Optional timeout for individual operation attempts
- `read_timeout`: Optional timeout for reading responses

### `make_client(endpoint_url: Option<String>, timeout_config: Option<TimeoutConfig>) -> Client`

Creates an SQS client with optional custom endpoint URL and timeout configuration. This is the most flexible option when you need fine-grained control over timeout settings.

## Usage

### Creating a Client

```rust
use aws_utils_sqs::make_client_with_timeout_default;

#[tokio::main]
async fn main() {
    // Create a client with default timeout configuration
    let client = make_client_with_timeout_default(None).await;
    
    // Or with a custom endpoint (e.g., for LocalStack)
    let client = make_client_with_timeout_default(Some("http://localhost:4566".to_string())).await;
}
```

### Creating a Client with Custom Timeouts

```rust
use std::time::Duration;
use aws_utils_sqs::make_client_with_timeout;

#[tokio::main]
async fn main() {
    // Create a client with custom timeout settings
    let client = make_client_with_timeout(
        None,
        Some(Duration::from_secs(5)),      // 5 second connect timeout
        Some(Duration::from_secs(30)),     // 30 second operation timeout
        Some(Duration::from_secs(25)),     // 25 second operation attempt timeout
        Some(Duration::from_secs(20)),     // 20 second read timeout
    ).await;
}
```

### Using TimeoutConfig

```rust
use aws_config::timeout::{TimeoutConfig, TimeoutConfigBuilder};
use aws_utils_sqs::make_client;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Build custom timeout configuration
    let timeout_config = TimeoutConfigBuilder::new()
        .connect_timeout(Duration::from_secs(10))
        .operation_timeout(Duration::from_secs(120))
        .build();
    
    // Create client with custom timeout configuration
    let client = make_client(None, Some(timeout_config)).await;
}
```

### Creating a Queue

```rust
use aws_utils_sqs::{sqs, builder::create_queue_attribute_builder::CreateQueueAttributeBuilder};

// Create a standard queue
let attributes = CreateQueueAttributeBuilder::new()
    .visibility_timeout(300)?
    .message_retention_period(345600)?
    .build()?;

let result = sqs::create_queue(&client, "my-queue", attributes, None).await?;
println!("Queue URL: {}", result.queue_url().unwrap());

// Create a FIFO queue with content-based deduplication
let attributes = CreateQueueAttributeBuilder::new()
    .content_based_deduplication(true)
    .fifo_throughput_limit(FifoThroughputLimit::PerMessageGroupId)
    .deduplication_scope(DeduplicationScope::MessageGroup)
    .build()?;

let result = sqs::create_queue(&client, "my-queue.fifo", attributes, None).await?;
```

### Sending Messages

```rust
use aws_utils_sqs::{sqs, builder::send_message_batch_entries_builder::SendMessageBatchEntriesBuilder};

// Send a single message
let result = sqs::send_message(
    &client,
    &queue_url,
    Some("Hello, SQS!".to_string()),
    None, // message_group_id (for FIFO queues)
    None, // message_deduplication_id
    None, // delay_seconds
    None, // message_attributes
    None, // message_system_attributes
).await?;

// Send messages in batch
let entries = SendMessageBatchEntriesBuilder::new()
    .add_message("msg1", "First message")
    .add_message_with_delay("msg2", "Delayed message", 60)
    .add_fifo_message("msg3", "FIFO message", "group1", Some("dedup1".to_string()))
    .build()?;

let result = sqs::send_message_batch(&client, &queue_url, entries).await?;
```

### Receiving Messages

```rust
// Receive up to 10 messages with long polling
let result = sqs::receive_message(
    &client,
    &queue_url,
    Some(10),                    // max_number_of_messages
    None,                        // message_attribute_names
    None,                        // message_system_attribute_names
    None,                        // receive_request_attempt_id
    None,                        // visibility_timeout
    Some(20),                    // wait_time_seconds (long polling)
).await?;

if let Some(messages) = result.messages() {
    for message in messages {
        println!("Message: {:?}", message.body());
        // Process message...
    }
}
```

### Deleting Messages

```rust
use aws_utils_sqs::builder::delete_message_batch_entries_builder::DeleteMessageBatchEntriesBuilder;

// Delete a single message
sqs::delete_message(&client, &queue_url, receipt_handle).await?;

// Delete messages in batch
let entries = DeleteMessageBatchEntriesBuilder::new()
    .add_message("msg1", receipt_handle1)
    .add_message("msg2", receipt_handle2)
    .build()?;

let result = sqs::delete_message_batch(&client, &queue_url, entries).await?;
```

### Working with Dead Letter Queues

```rust
use aws_utils_sqs::builder::create_queue_attribute_builder::{RedrivePolicy, RedriveAllowPolicy};

// Configure a dead letter queue
let redrive_policy = RedrivePolicy::new(5, dead_letter_queue_arn);

let attributes = CreateQueueAttributeBuilder::new()
    .redrive_policy(redrive_policy)
    .build()?;

// Configure which queues can use this queue as a dead letter queue
let redrive_allow_policy = RedriveAllowPolicy::by_queue(vec![
    source_queue_arn1.to_string(),
    source_queue_arn2.to_string(),
]);

let attributes = CreateQueueAttributeBuilder::new()
    .redrive_allow_policy(redrive_allow_policy)
    .build()?;
```

## Error Handling

The library uses a custom `Error` type that wraps AWS SDK errors and provides additional context:

```rust
use aws_utils_sqs::sqs::Error;

match sqs::create_queue(&client, "my-queue", attributes, None).await {
    Ok(output) => println!("Queue created: {:?}", output.queue_url()),
    Err(Error::AwsSdkError(e)) => eprintln!("AWS SDK error: {}", e),
    Err(Error::ValidationError(e)) => eprintln!("Validation error: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.