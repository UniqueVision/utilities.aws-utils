use aws_sdk_s3::{
    Client,
    operation::{create_bucket::CreateBucketOutput, delete_bucket::DeleteBucketOutput},
    types::Bucket,
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{TryStream, TryStreamExt};

use crate::{
    error::{Error, from_aws_sdk_error},
    object::delete_objects,
};

pub async fn create_bucket(
    client: &Client,
    bucket_name: impl Into<String>,
) -> Result<CreateBucketOutput, Error> {
    client
        .create_bucket()
        .bucket(bucket_name)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub fn list_stream(
    client: &Client,
    prefix: impl Into<String>,
) -> impl TryStream<Ok = Bucket, Error = Error> {
    client
        .list_buckets()
        .prefix(prefix.into())
        .into_paginator()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
        .map_ok(|s| futures_util::stream::iter(s.buckets.unwrap_or_default().into_iter().map(Ok)))
        .try_flatten()
}

pub async fn list_all(client: &Client, prefix: impl Into<String>) -> Result<Vec<Bucket>, Error> {
    list_stream(client, prefix).try_collect().await
}

pub async fn delete_bucket(
    client: &Client,
    bucket_name: impl Into<String>,
) -> Result<DeleteBucketOutput, Error> {
    let bucket_name = bucket_name.into();
    delete_objects(client, &bucket_name, None::<String>).await?;
    client
        .delete_bucket()
        .bucket(&bucket_name)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn delete_buckets(client: &Client, prefix: impl Into<String>) -> Result<(), Error> {
    let mut stream = list_stream(client, prefix);
    while let Some(bucket) = stream.try_next().await? {
        if let Some(name) = bucket.name() {
            delete_bucket(client, name).await?;
        }
    }
    Ok(())
}
