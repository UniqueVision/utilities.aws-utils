# aws_utils_scheduler

A Rust wrapper for AWS EventBridge Scheduler with type-safe builders for schedule expressions.

[![Crates.io](https://img.shields.io/crates/v/aws_utils_scheduler.svg)](https://crates.io/crates/aws_utils_scheduler)
[![Documentation](https://docs.rs/aws_utils_scheduler/badge.svg)](https://docs.rs/aws_utils_scheduler)
[![License](https://img.shields.io/crates/l/aws_utils_scheduler.svg)](LICENSE)

## Overview

`aws_utils_scheduler` provides a convenient and type-safe interface for working with AWS EventBridge Scheduler. It includes:

- Simple client creation with optional endpoint configuration
- Type-safe builders for schedule expressions (at, rate, cron)
- Stream-based pagination for listing schedules
- Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aws_utils_scheduler = "0.1.0"
```

## Usage

### Creating a Client

```rust
use aws_utils_scheduler;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default timeout configuration
    let client = aws_utils_scheduler::make_client_with_timeout_default(None).await;
    
    // Or with a custom endpoint and default timeouts
    let client = aws_utils_scheduler::make_client_with_timeout_default(
        Some("http://localhost:4566".to_string())
    ).await;
    
    // Or with custom timeout settings
    let client = aws_utils_scheduler::make_client_with_timeout(
        None,
        Some(Duration::from_secs(30)),   // connect timeout
        Some(Duration::from_secs(120)),  // operation timeout
        Some(Duration::from_secs(60)),   // operation attempt timeout
        Some(Duration::from_secs(30)),   // read timeout
    ).await;
    
    // Or without timeout configuration
    let client = aws_utils_scheduler::make_client(None, None).await;
    
    Ok(())
}
```

### Creating Schedules

#### One-time Schedule (At Expression)

```rust
use aws_utils_scheduler::{scheduler, builder::AtExpressionBuilder};
use aws_sdk_scheduler::types::{Target, FlexibleTimeWindow, FlexibleTimeWindowMode};
use chrono::{Utc, Duration};

let future_time = Utc::now() + Duration::hours(1);
let at_expression = AtExpressionBuilder::new(future_time).build()?;

let target = Target::builder()
    .arn("arn:aws:lambda:us-east-1:123456789012:function:MyFunction")
    .role_arn("arn:aws:iam::123456789012:role/MyRole")
    .build()
    .unwrap();

let flexible_window = FlexibleTimeWindow::builder()
    .mode(FlexibleTimeWindowMode::Off)
    .build()
    .unwrap();

scheduler::create_schedule(
    &client,
    "my-schedule",
    None,  // group_name
    &at_expression,
    None,  // start_date
    None,  // end_date
    None,  // description
    None,  // timezone
    None,  // state
    None,  // kms_key_arn
    Some(target),
    Some(flexible_window),
    None,  // client_token
    None,  // action_after_completion
).await?;
```

#### Recurring Schedule (Rate Expression)

```rust
use aws_utils_scheduler::builder::{RateExpressionBuilder, RateUnit};
use aws_sdk_scheduler::types::{Target, FlexibleTimeWindow, FlexibleTimeWindowMode};

let rate_expression = RateExpressionBuilder::new(5, RateUnit::Minutes).build()?;

let target = Target::builder()
    .arn("arn:aws:lambda:us-east-1:123456789012:function:MyFunction")
    .role_arn("arn:aws:iam::123456789012:role/MyRole")
    .build()
    .unwrap();

let flexible_window = FlexibleTimeWindow::builder()
    .mode(FlexibleTimeWindowMode::Off)
    .build()
    .unwrap();

scheduler::create_schedule(
    &client,
    "my-recurring-schedule",
    None,  // group_name
    &rate_expression,
    None,  // start_date
    None,  // end_date
    None,  // description
    None,  // timezone
    None,  // state
    None,  // kms_key_arn
    Some(target),
    Some(flexible_window),
    None,  // client_token
    None,  // action_after_completion
).await?;
```

#### Cron Schedule

```rust
use aws_utils_scheduler::builder::CronExpressionBuilder;
use aws_sdk_scheduler::types::{Target, FlexibleTimeWindow, FlexibleTimeWindowMode};

let cron_expression = CronExpressionBuilder::new()
    .minutes("0")
    .hours("12")
    .days_of_month("*")
    .months("*")
    .days_of_week("MON-FRI")
    .build()?;

let target = Target::builder()
    .arn("arn:aws:lambda:us-east-1:123456789012:function:MyFunction")
    .role_arn("arn:aws:iam::123456789012:role/MyRole")
    .build()
    .unwrap();

let flexible_window = FlexibleTimeWindow::builder()
    .mode(FlexibleTimeWindowMode::Off)
    .build()
    .unwrap();

scheduler::create_schedule(
    &client,
    "weekday-noon-schedule",
    None,  // group_name
    &cron_expression,
    None,  // start_date
    None,  // end_date
    None,  // description
    None,  // timezone
    None,  // state
    None,  // kms_key_arn
    Some(target),
    Some(flexible_window),
    None,  // client_token
    None,  // action_after_completion
).await?;
```

### Listing Schedules

#### Stream-based Listing

```rust
use futures_util::TryStreamExt;

let stream = scheduler::list_schedules_stream(
    &client,
    None::<String>,  // name_prefix
    None::<String>,  // group_name
    None,            // state
);
futures_util::pin_mut!(stream);

while let Some(schedule) = stream.try_next().await? {
    println!("Schedule: {:?}", schedule.name());
}
```

#### Batch Listing

```rust
let schedules = scheduler::list_schedules_all(
    &client,
    None::<String>,  // name_prefix
    None::<String>,  // group_name
    None,            // state
).await?;
for schedule in schedules {
    println!("Schedule: {:?}", schedule.name());
}
```

### Other Operations

```rust
use aws_sdk_scheduler::types::{Target, FlexibleTimeWindow, FlexibleTimeWindowMode};

// Get schedule details
let schedule = scheduler::get_scheduler(
    &client,
    "my-schedule",
    None::<String>,  // group_name
).await?;

// Update a schedule
let target = Target::builder()
    .arn("arn:aws:lambda:us-east-1:123456789012:function:NewFunction")
    .role_arn("arn:aws:iam::123456789012:role/MyRole")
    .build()
    .unwrap();

let flexible_window = FlexibleTimeWindow::builder()
    .mode(FlexibleTimeWindowMode::Off)
    .build()
    .unwrap();

scheduler::update_schedule(
    &client,
    "my-schedule",
    None,  // group_name
    &new_expression,
    None,  // start_date
    None,  // end_date
    None,  // description
    None,  // timezone
    None,  // state
    None,  // kms_key_arn
    Some(target),
    Some(flexible_window),
    None,  // client_token
    None,  // action_after_completion
).await?;

// Delete a schedule
scheduler::delete_schedule(
    &client,
    "my-schedule",
    None::<String>,  // group_name
    None::<String>,  // client_token
).await?;
```

## Schedule Expression Builders

### AtExpressionBuilder

Creates one-time schedules that run at a specific date and time.

```rust
use chrono::Utc;
let at_expr = AtExpressionBuilder::new(Utc::now() + Duration::days(1)).build()?;
// Returns: "at(2024-01-02T15:30:00)"
```

### RateExpressionBuilder

Creates recurring schedules that run at regular intervals.

```rust
let rate_expr = RateExpressionBuilder::new(30, RateUnit::Minutes).build()?;
// Returns: "rate(30 minutes)"
```

### CronExpressionBuilder

Creates schedules using cron expressions for complex timing requirements.

```rust
let cron_expr = CronExpressionBuilder::new()
    .minutes("0")
    .hours("9")
    .days_of_month("*")
    .months("*")
    .days_of_week("MON-FRI")
    .build()?;
// Returns: "cron(0 9 * * MON-FRI)"
```

## Error Handling

The crate provides comprehensive error handling through the `SchedulerError` enum:

```rust
use aws_utils_scheduler::error::SchedulerError;

match scheduler::create_schedule(
    &client,
    name,
    group,
    expr,
    None,  // start_date
    None,  // end_date
    None,  // description
    None,  // timezone
    None,  // state
    None,  // kms_key_arn
    Some(target),
    Some(flexible_window),
    None,  // client_token
    None,  // action_after_completion
).await {
    Ok(_) => println!("Schedule created successfully"),
    Err(SchedulerError::Aws(e)) => eprintln!("AWS error: {}", e),
    Err(SchedulerError::InvalidScheduleExpression) => eprintln!("Invalid expression"),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Important Notes

- All client creation functions (`make_client`, `make_client_with_timeout`, `make_client_with_timeout_default`) set dummy AWS credentials if they're not already present in the environment. This is useful for local development with tools like LocalStack.
- The `make_client_with_timeout_default` function provides reasonable default timeout values (connect: 3100s, operation: 60s, operation attempt: 55s, read: 50s) suitable for most use cases.
- All schedule names must be unique within a schedule group.
- The IAM role must have the necessary permissions to invoke the target.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.