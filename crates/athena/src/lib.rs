use std::time::Duration;

use aws_config::{
    BehaviorVersion,
    timeout::{TimeoutConfig, TimeoutConfigBuilder},
};
use aws_sdk_athena::Client;

pub mod error;
pub mod query;
pub mod stream;

pub use aws_sdk_athena;

pub async fn make_client_with_timeout_default(endpoint_url: Option<String>) -> Client {
    make_client_with_timeout(
        endpoint_url,
        Duration::from_secs(60),
        Duration::from_secs(55),
        Duration::from_secs(50),
    )
    .await
}

pub async fn make_client_with_timeout(
    endpoint_url: Option<String>,
    operation_timeout: Duration,
    operation_attempt_timeout: Duration,
    connect_timeout: Duration,
) -> Client {
    let timeout_config = TimeoutConfigBuilder::new()
        .operation_timeout(operation_timeout)
        .operation_attempt_timeout(operation_attempt_timeout)
        .connect_timeout(connect_timeout)
        .build();
    make_client(endpoint_url, Some(timeout_config)).await
}

pub async fn make_client(
    endpoint_url: Option<String>,
    timeout_config: Option<TimeoutConfig>,
) -> Client {
    if std::env::var("AWS_ACCESS_KEY_ID").is_err() {
        unsafe { std::env::set_var("AWS_ACCESS_KEY_ID", "dummy_access_key") };
    }
    if std::env::var("AWS_SECRET_ACCESS_KEY").is_err() {
        unsafe { std::env::set_var("AWS_SECRET_ACCESS_KEY", "dummy_secret_key") };
    }
    if std::env::var("AWS_REGION").is_err() {
        unsafe { std::env::set_var("AWS_REGION", "us-west-2") };
    }
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
    if let Some(timeout_config) = timeout_config {
        config_loader = config_loader.timeout_config(timeout_config);
    }
    let config = config_loader.load().await;
    let mut builder = aws_sdk_athena::config::Builder::from(&config);
    if let Some(aws_endpoint_url) = endpoint_url {
        builder = builder.endpoint_url(aws_endpoint_url)
    }
    Client::from_conf(builder.build())
}
