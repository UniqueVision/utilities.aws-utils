use std::time::Duration;

use aws_config::{
    BehaviorVersion,
    timeout::{TimeoutConfig, TimeoutConfigBuilder},
};
use aws_sdk_sqs::Client;

pub mod builder;
pub mod error;
pub mod sqs;

pub use aws_sdk_sqs;

pub async fn make_client_with_timeout_default(endpoint_url: Option<String>) -> Client {
    make_client_with_timeout(
        endpoint_url,
        Some(Duration::from_secs(3100)),
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(55)),
        Some(Duration::from_secs(50)),
    )
    .await
}

pub async fn make_client_with_timeout(
    endpoint_url: Option<String>,
    connect_timeout: Option<Duration>,
    operation_timeout: Option<Duration>,
    operation_attempt_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
) -> Client {
    let mut timeout_config = TimeoutConfigBuilder::new();
    timeout_config
        .set_connect_timeout(connect_timeout)
        .set_operation_timeout(operation_timeout)
        .set_operation_attempt_timeout(operation_attempt_timeout)
        .set_read_timeout(read_timeout);
    make_client(endpoint_url, Some(timeout_config.build())).await
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
    let mut builder = aws_sdk_sqs::config::Builder::from(&config);
    if let Some(aws_endpoint_url) = endpoint_url {
        builder = builder.endpoint_url(aws_endpoint_url)
    }
    Client::from_conf(builder.build())
}
