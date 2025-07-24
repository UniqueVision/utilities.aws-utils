use std::{path::Path, time::Duration};

use aws_sdk_s3::{
    Client,
    operation::{
        delete_object::DeleteObjectOutput, get_object::GetObjectOutput, put_object::PutObjectOutput,
    },
    presigning::{PresignedRequest, PresigningConfig},
    primitives::ByteStream,
    types::Object,
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{TryStream, TryStreamExt};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::error::{Error, from_aws_sdk_error};

pub fn list_stream(
    client: &Client,
    bucket_name: impl Into<String>,
    prefix: impl Into<String>,
) -> impl TryStream<Ok = Object, Error = Error> {
    client
        .list_objects_v2()
        .bucket(bucket_name.into())
        .prefix(prefix.into())
        .into_paginator()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
        .map_ok(|s| futures_util::stream::iter(s.contents.unwrap_or_default().into_iter().map(Ok)))
        .try_flatten()
}

pub async fn list_all(
    client: &Client,
    bucket_name: impl Into<String>,
    prefix: impl Into<String>,
) -> Result<Vec<Object>, Error> {
    list_stream(client, bucket_name, prefix).try_collect().await
}

pub async fn get_object(
    client: &Client,
    bucket_name: impl Into<String>,
    key: impl Into<String>,
) -> Result<GetObjectOutput, Error> {
    client
        .get_object()
        .bucket(bucket_name.into())
        .key(key.into())
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_object_string(object: GetObjectOutput) -> Result<(String, String), Error> {
    let content_type = object.content_type().unwrap_or_default().to_string();
    let mut reader = get_object_buf_reader(object);
    let mut dst = String::new();
    reader.read_to_string(&mut dst).await?;
    Ok((content_type, dst))
}

pub fn get_object_buf_reader(object: GetObjectOutput) -> BufReader<impl AsyncRead> {
    BufReader::new(object.body.into_async_read())
}

pub async fn put_object(
    client: &Client,
    bucket_name: impl Into<String>,
    key: impl Into<String>,
    body: impl Into<ByteStream>,
    content_type: Option<impl Into<String>>,
    content_disposition: Option<impl Into<String>>,
) -> Result<PutObjectOutput, Error> {
    client
        .put_object()
        .set_bucket(Some(bucket_name.into()))
        .set_key(Some(key.into()))
        .set_body(Some(body.into()))
        .set_content_type(content_type.map(Into::into))
        .set_content_disposition(content_disposition.map(Into::into))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn put_object_from_path(
    client: &Client,
    bucket_name: impl Into<String>,
    key: impl Into<String>,
    file_path: impl AsRef<Path>,
    content_type: Option<impl Into<String>>,
    content_disposition: Option<impl Into<String>>,
) -> Result<PutObjectOutput, Error> {
    put_object(
        client,
        bucket_name,
        key,
        ByteStream::from_path(file_path).await?,
        content_type,
        content_disposition,
    )
    .await
}

pub async fn delete_object(
    client: &Client,
    bucket_name: impl Into<String>,
    key: impl Into<String>,
) -> Result<DeleteObjectOutput, Error> {
    client
        .delete_object()
        .set_bucket(Some(bucket_name.into()))
        .set_key(Some(key.into()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

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

pub async fn delete_objects(
    client: &Client,
    bucket_name: impl Into<String>,
    prefix: impl Into<String>,
) -> Result<(), Error> {
    let batch_size = 1000;
    let bucket_name = bucket_name.into();
    let mut stream = list_stream(client, &bucket_name, prefix);
    let mut delete_object_ids: Vec<aws_sdk_s3::types::ObjectIdentifier> = vec![];
    while let Some(object) = stream.try_next().await? {
        if let Some(key) = object.key() {
            delete_object_ids.push(
                aws_sdk_s3::types::ObjectIdentifier::builder()
                    .key(key.to_owned())
                    .build()?,
            );
            if delete_object_ids.len() >= batch_size as usize {
                // 1000個以上の削除リクエストはエラーになるので、1000個ごとに削除リクエストを送る
                client
                    .delete_objects()
                    .bucket(&bucket_name)
                    .delete(
                        aws_sdk_s3::types::Delete::builder()
                            .set_objects(Some(delete_object_ids))
                            .build()?,
                    )
                    .send()
                    .await
                    .map_err(from_aws_sdk_error)?;
                delete_object_ids = vec![];
            }
        }
    }
    // 1000個未満の削除リクエストを送る
    if !delete_object_ids.is_empty() {
        client
            .delete_objects()
            .bucket(&bucket_name)
            .delete(
                aws_sdk_s3::types::Delete::builder()
                    .set_objects(Some(delete_object_ids))
                    .build()?,
            )
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
    }
    Ok(())
}
