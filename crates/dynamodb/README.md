# aws_utils_dynamodb

AWS DynamoDB utilities for Rust, providing a simplified interface for common DynamoDB operations.

## Features

- Simple DynamoDB client creation with configurable endpoint
- Record operations (CRUD)
- Table management operations
- Stream-based pagination for scan and query operations
- CSV import functionality from S3
- Error handling with custom error types

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_dynamodb = { path = "crates/dynamodb" }
```

## Usage

### Creating a Client

```rust
use aws_utils_dynamodb::{make_client, make_client_with_timeout_default, make_client_with_timeout};
use std::time::Duration;

// Create client with default timeout settings
let client = make_client_with_timeout_default(None).await;

// Create client with custom timeout settings
let client = make_client_with_timeout(
    None, // endpoint_url
    Some(Duration::from_secs(3100)), // connect_timeout
    Some(Duration::from_secs(60)),   // operation_timeout
    Some(Duration::from_secs(55)),   // operation_attempt_timeout
    Some(Duration::from_secs(50)),   // read_timeout
).await;

// Create client with custom endpoint and default timeout
let client = make_client_with_timeout_default(
    Some("http://localhost:8000".to_string())
).await;

// Create client without timeout configuration (legacy)
let client = make_client(None, None).await;

// Create client with custom endpoint and no timeout (legacy)
let client = make_client(Some("http://localhost:8000".to_string()), None).await;
```

### Record Operations

```rust
use aws_utils_dynamodb::record::{get_item, put_item, update_item, delete_item, scan_all, query_all};
use aws_sdk_dynamodb::types::{AttributeValue, ReturnValue};
use std::collections::HashMap;

// Get an item
let mut key = HashMap::new();
key.insert("id".to_string(), AttributeValue::S("123".to_string()));
let item = get_item(&client, "my_table", key).await?;

// Put an item
let mut item = HashMap::new();
item.insert("id".to_string(), AttributeValue::S("123".to_string()));
item.insert("name".to_string(), AttributeValue::S("John".to_string()));
let output = put_item(&client, "my_table", item, None, None, None, None).await?;

// Update an item
let mut key = HashMap::new();
key.insert("id".to_string(), AttributeValue::S("123".to_string()));
let output = update_item(
    &client,
    "my_table",
    key,
    "SET #name = :name",
    None,
    Some(HashMap::from([("#name".to_string(), "name".to_string())])),
    Some(HashMap::from([(":name".to_string(), AttributeValue::S("Jane".to_string()))])),
    Some(ReturnValue::AllNew)
).await?;

// Delete an item
let mut key = HashMap::new();
key.insert("id".to_string(), AttributeValue::S("123".to_string()));
let output = delete_item(&client, "my_table", key, None, None, None, None).await?;

// Scan all items
let items = scan_all(&client, "my_table", None, None, None, None).await?;

// Query items
let items = query_all(
    &client,
    "my_table",
    None,
    Some("id = :id"),
    None,
    None,
    Some(HashMap::from([(":id".to_string(), AttributeValue::S("123".to_string()))]))
).await?;
```

### Table Operations

```rust
use aws_utils_dynamodb::table::{create_table, delete_table, describe_table, get_capacity, set_capacity, TableType};
use aws_sdk_dynamodb::types::{AttributeDefinition, ScalarAttributeType};

// Create a table with on-demand billing
let attrs = vec![
    AttributeDefinition::builder()
        .attribute_name("id")
        .attribute_type(ScalarAttributeType::S)
        .build()?,
];
let output = create_table(
    &client,
    "my_table",
    "id",
    None::<String>,
    TableType::OnDemand,
    attrs,
    None
).await?;

// Create a table with provisioned capacity
let output = create_table(
    &client,
    "my_table",
    "id",
    Some("timestamp"),
    TableType::Provisioned(5, 5), // 5 RCU, 5 WCU
    attrs,
    None
).await?;

// Delete a table
let output = delete_table(&client, "my_table").await?;

// Get table capacity
let (read_units, write_units) = get_capacity(&client, "my_table").await?;

// Update table capacity
let output = set_capacity(&client, "my_table", 10, 10).await?;
```

### CSV Import from S3

```rust
use aws_utils_dynamodb::csv::import_table;
use aws_utils_dynamodb::table::TableType;
use aws_sdk_dynamodb::types::{AttributeDefinition, ScalarAttributeType};

// Import CSV data from S3 to a new DynamoDB table
let attrs = vec![
    AttributeDefinition::builder()
        .attribute_name("id")
        .attribute_type(ScalarAttributeType::S)
        .build()?,
];

import_table(
    &client,
    "my-bucket",
    "data/users.csv",
    Some(","), // delimiter
    Some(vec!["id".to_string(), "name".to_string(), "email".to_string()]), // headers
    "imported_users_table",
    "id", // hash key
    None::<String>, // no sort key
    attrs,
    TableType::OnDemand
).await?;
```

### Stream Operations

For handling large datasets, use stream-based operations:

```rust
use aws_utils_dynamodb::record::{scan_stream, query_stream};
use futures_util::TryStreamExt;

// Scan with streaming
let stream = scan_stream(&client, "my_table", None, None, None, None);
futures_util::pin_mut!(stream);
while let Some(item) = stream.try_next().await? {
    // Process each item
    println!("{:?}", item);
}

// Query with streaming
let stream = query_stream(
    &client,
    "my_table",
    None,
    Some("id = :id"),
    None,
    None,
    Some(HashMap::from([(":id".to_string(), AttributeValue::S("123".to_string()))]))
);
futures_util::pin_mut!(stream);
while let Some(item) = stream.try_next().await? {
    // Process each item
    println!("{:?}", item);
}
```

## Error Handling

The crate provides a custom `Error` type that wraps AWS SDK errors and includes common error cases:

- `NotFound` - Item not found
- `ValidationError` - Invalid parameters or state
- `Invalid` - Invalid response from AWS
- `AwsSdkError` - AWS SDK specific errors

## Environment Variables

The client creation automatically sets default values for AWS credentials if not present:
- `AWS_ACCESS_KEY_ID` - Defaults to "dummy_access_key"
- `AWS_SECRET_ACCESS_KEY` - Defaults to "dummy_secret_key"
- `AWS_REGION` - Defaults to "us-west-2"

These defaults are useful for local development with DynamoDB Local.

## License

This project is part of the utilities.aws-utils workspace.