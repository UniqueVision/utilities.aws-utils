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
use aws_utils_scheduler::scheduler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default AWS configuration
    let client = scheduler::make_client(None).await;
    
    // Or with a custom endpoint
    let client = scheduler::make_client(Some("http://localhost:4566".to_string())).await;
    
    Ok(())
}
```

### Creating Schedules

#### One-time Schedule (At Expression)

```rust
use aws_utils_scheduler::{scheduler, builder::AtExpressionBuilder};
use chrono::{Utc, Duration};

let future_time = Utc::now() + Duration::hours(1);
let at_expression = AtExpressionBuilder::new(future_time).build()?;

scheduler::create_schedule(
    &client,
    "my-schedule",
    &at_expression,
    "arn:aws:lambda:us-east-1:123456789012:function:MyFunction",
    "arn:aws:iam::123456789012:role/MyRole",
    None,
).await?;
```

#### Recurring Schedule (Rate Expression)

```rust
use aws_utils_scheduler::builder::{RateExpressionBuilder, RateUnit};

let rate_expression = RateExpressionBuilder::new(5, RateUnit::Minutes).build()?;

scheduler::create_schedule(
    &client,
    "my-recurring-schedule",
    &rate_expression,
    "arn:aws:lambda:us-east-1:123456789012:function:MyFunction",
    "arn:aws:iam::123456789012:role/MyRole",
    None,
).await?;
```

#### Cron Schedule

```rust
use aws_utils_scheduler::builder::CronExpressionBuilder;

let cron_expression = CronExpressionBuilder::new()
    .minutes("0")
    .hours("12")
    .days_of_month("*")
    .months("*")
    .days_of_week("MON-FRI")
    .build()?;

scheduler::create_schedule(
    &client,
    "weekday-noon-schedule",
    &cron_expression,
    "arn:aws:lambda:us-east-1:123456789012:function:MyFunction",
    "arn:aws:iam::123456789012:role/MyRole",
    None,
).await?;
```

### Listing Schedules

#### Stream-based Listing

```rust
use futures_util::TryStreamExt;

let stream = scheduler::list_schedules_stream(&client, None, None);
futures_util::pin_mut!(stream);

while let Some(schedule) = stream.try_next().await? {
    println!("Schedule: {:?}", schedule.name());
}
```

#### Batch Listing

```rust
let schedules = scheduler::list_schedules(&client, None, None, None).await?;
for schedule in schedules {
    println!("Schedule: {:?}", schedule.name());
}
```

### Other Operations

```rust
// Get schedule details
let schedule = scheduler::get_schedule(&client, "my-schedule", None).await?;

// Update a schedule
scheduler::update_schedule(
    &client,
    "my-schedule",
    &new_expression,
    "arn:aws:lambda:us-east-1:123456789012:function:NewFunction",
    "arn:aws:iam::123456789012:role/MyRole",
    None,
).await?;

// Delete a schedule
scheduler::delete_schedule(&client, "my-schedule", None).await?;
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

match scheduler::create_schedule(&client, name, expr, target, role, group).await {
    Ok(_) => println!("Schedule created successfully"),
    Err(SchedulerError::Aws(e)) => eprintln!("AWS error: {}", e),
    Err(SchedulerError::InvalidScheduleExpression) => eprintln!("Invalid expression"),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Important Notes

- The `make_client` function sets dummy AWS credentials if they're not already present in the environment. This is useful for local development with tools like LocalStack.
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