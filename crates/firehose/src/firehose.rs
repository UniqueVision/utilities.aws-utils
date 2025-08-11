use aws_sdk_firehose::{operation::put_record::PutRecordOutput, types::Record, Client};
use crate::error::{from_aws_sdk_error, Error};

pub async fn put_record(client: &Client, delivery_stream_name: impl Into<String>, data: impl Into<Vec<u8>>) -> Result<PutRecordOutput, Error> {
    let record = Record::builder()
        .data(data.into().into())
        .build()?;

    client
        .put_record()
        .delivery_stream_name(delivery_stream_name.into())
        .record(record)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}