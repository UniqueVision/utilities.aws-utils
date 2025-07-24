use aws_sdk_s3::{Client, types::Object};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{TryStream, TryStreamExt};

use crate::error::{Error, from_aws_sdk_error};

pub fn get_stream(
    client: &Client,
    bucket_name: Option<impl Into<String>>,
    prefix: Option<impl Into<String>>,
) -> impl TryStream<Ok = Object, Error = Error> {
    client
        .list_objects_v2()
        .set_bucket(bucket_name.map(Into::into))
        .set_prefix(prefix.map(Into::into))
        .into_paginator()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
        .map_ok(|s| futures_util::stream::iter(s.contents.unwrap_or_default().into_iter().map(Ok)))
        .try_flatten()
}
