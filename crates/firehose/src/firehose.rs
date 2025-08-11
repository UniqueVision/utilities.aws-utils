use crate::error::{Error, from_aws_sdk_error};
use aws_sdk_firehose::{Client, operation::put_record::PutRecordOutput, types::Record};

pub async fn put_record(
    client: &Client,
    delivery_stream_name: impl Into<String>,
    data: impl Into<Vec<u8>>,
) -> Result<PutRecordOutput, Error> {
    let record = Record::builder().data(data.into().into()).build()?;

    client
        .put_record()
        .delivery_stream_name(delivery_stream_name.into())
        .record(record)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}
