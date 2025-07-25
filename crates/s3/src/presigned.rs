use std::time::Duration;

use aws_sdk_s3::{
    Client,
    presigning::{PresignedRequest, PresigningConfig},
};

use crate::error::{Error, from_aws_sdk_error};

pub async fn put_presigned(
    client: &Client,
    bucket_name: impl Into<String>,
    key: impl Into<String>,
    duration: Duration,
) -> Result<PresignedRequest, Error> {
    client
        .put_object()
        .set_bucket(Some(bucket_name.into()))
        .set_key(Some(key.into()))
        .presigned(PresigningConfig::expires_in(duration)?)
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_presigned(
    client: &Client,
    bucket_name: impl Into<String>,
    key: impl Into<String>,
    duration: Duration,
) -> Result<PresignedRequest, Error> {
    client
        .get_object()
        .set_bucket(Some(bucket_name.into()))
        .set_key(Some(key.into()))
        .presigned(PresigningConfig::expires_in(duration)?)
        .await
        .map_err(from_aws_sdk_error)
}

pub fn presigned_url(presigned_request: &PresignedRequest) -> String {
    presigned_request.uri().to_string()
}
