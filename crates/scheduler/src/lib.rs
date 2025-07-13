pub async fn make_client(endpoint_url: Option<String>) -> aws_sdk_scheduler::Client {
    if std::env::var("AWS_ACCESS_KEY_ID").is_err() {
        unsafe { std::env::set_var("AWS_ACCESS_KEY_ID", "dummy_access_key") };
    }
    if std::env::var("AWS_SECRET_ACCESS_KEY").is_err() {
        unsafe { std::env::set_var("AWS_SECRET_ACCESS_KEY", "dummy_secret_key") };
    }
    if std::env::var("AWS_REGION").is_err() {
        unsafe { std::env::set_var("AWS_REGION", "us-west-2") };
    }
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let mut builder = aws_sdk_scheduler::config::Builder::from(&config);
    if let Some(aws_endpoint_url) = endpoint_url {
        builder = builder.endpoint_url(aws_endpoint_url)
    }
    aws_sdk_scheduler::Client::from_conf(builder.build())
}
