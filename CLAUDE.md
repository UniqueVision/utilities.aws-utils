# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project providing AWS utilities, currently focused on Kinesis Data Streams functionality. The project is in early development stage with incomplete dependencies.

## Project Structure

- `/crates/kinesis_data_streams/` - Main crate implementing Kinesis Data Streams client
  - `error.rs` - Error types using thiserror
  - `kinesis_data_stream.rs` - AWS Kinesis client implementation
  - `lib.rs` - Public API and Records struct for batch operations

## Important Notes

The project currently references workspace dependencies but lacks a root `Cargo.toml`. Additionally, it depends on two local crates that don't exist in the repository:
- `logs = { path = "../logs" }`
- `retry_utils = { path = "../retry_utils" }`

These missing dependencies must be addressed before the project can build successfully.

## Development Commands

### Testing
Run tests for the kinesis_data_streams crate:
```bash
RUST_LOG=info REALM_CODE=test cargo test -p kinesis_data_streams test_kinesis_data_streams_records -- --nocapture --test-threads=1
```

Note: The test requires `REALM_CODE` environment variable to be set.

### Standard Rust Commands
```bash
cargo build          # Build the project (requires fixing missing dependencies first)
cargo test           # Run all tests
cargo fmt            # Format code
cargo clippy         # Run linter
```

## Architecture

The codebase implements a client for AWS Kinesis Data Streams with:
- Batch record processing via the `Records` struct
- Retry logic for put_records operations
- Error handling with custom error types
- Async operations using tokio runtime
- Support for both simple and aggregated record formats

The client provides methods for:
- Creating data streams
- Deleting data streams  
- Putting individual records
- Batch putting records with automatic retry on failures

Code includes Japanese comments documenting AWS service limits and restrictions.

## Note
- Cargoのeditionは2024です。