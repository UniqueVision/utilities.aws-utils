use aws_sdk_kinesis::{
    operation::{put_record::PutRecordOutput, put_records::PutRecordsOutput},
    primitives::Blob,
    types::PutRecordsRequestEntry,
};

use crate::error::{Error, from_aws_sdk_error};

pub async fn add_record(
    client: &aws_sdk_kinesis::Client,
    stream_name: impl Into<String>,
    partition_key: impl Into<String>,
    data: impl Into<Vec<u8>>,
) -> Result<PutRecordOutput, Error> {
    let blob = Blob::new(data);
    client
        .put_record()
        .stream_name(stream_name)
        .partition_key(partition_key)
        .data(blob)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn add_records(
    client: &aws_sdk_kinesis::Client,
    stream_name: impl Into<String>,
    records: Vec<PutRecordsRequestEntry>,
) -> Result<PutRecordsOutput, Error> {
    client
        .put_records()
        .stream_name(stream_name)
        .set_records(Some(records))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::make_client;
    use aws_sdk_kinesis::primitives::Blob;
    use mockito::Server;

    #[tokio::test]
    async fn test_add_record_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/")
            .match_header("content-type", "application/x-amz-json-1.1")
            .match_header("x-amz-target", "Kinesis_20131202.PutRecord")
            .with_status(200)
            .with_body(
                r#"{
                "SequenceNumber": "12345",
                "ShardId": "shardId-000000000000"
            }"#,
            )
            .create_async()
            .await;

        let client = make_client(Some(server.url())).await;
        let result = add_record(
            &client,
            "test-stream",
            "test-partition",
            "test-data".to_string(),
        )
        .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.sequence_number(), "12345");
        assert_eq!(output.shard_id(), "shardId-000000000000");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_add_record_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/")
            .match_header("content-type", "application/x-amz-json-1.1")
            .match_header("x-amz-target", "Kinesis_20131202.PutRecord")
            .with_status(400)
            .with_body(
                r#"{
                "__type": "ResourceNotFoundException",
                "message": "Stream test-stream under account 123456789012 not found."
            }"#,
            )
            .create_async()
            .await;

        let client = make_client(Some(server.url())).await;
        let result = add_record(
            &client,
            "test-stream",
            "test-partition",
            "test-data".to_string(),
        )
        .await;

        assert!(result.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_add_records_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/")
            .match_header("content-type", "application/x-amz-json-1.1")
            .match_header("x-amz-target", "Kinesis_20131202.PutRecords")
            .with_status(200)
            .with_body(
                r#"{
                "FailedRecordCount": 0,
                "Records": [
                    {
                        "SequenceNumber": "12345",
                        "ShardId": "shardId-000000000000"
                    },
                    {
                        "SequenceNumber": "12346",
                        "ShardId": "shardId-000000000001"
                    }
                ]
            }"#,
            )
            .create_async()
            .await;

        let client = make_client(Some(server.url())).await;

        let records = vec![
            PutRecordsRequestEntry::builder()
                .data(Blob::new("test-data-1"))
                .partition_key("partition-1")
                .build()
                .unwrap(),
            PutRecordsRequestEntry::builder()
                .data(Blob::new("test-data-2"))
                .partition_key("partition-2")
                .build()
                .unwrap(),
        ];

        let result = add_records(&client, "test-stream", records).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.failed_record_count(), Some(0));
        assert_eq!(output.records().len(), 2);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_add_records_partial_failure() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/")
            .match_header("content-type", "application/x-amz-json-1.1")
            .match_header("x-amz-target", "Kinesis_20131202.PutRecords")
            .with_status(200)
            .with_body(
                r#"{
                "FailedRecordCount": 1,
                "Records": [
                    {
                        "SequenceNumber": "12345",
                        "ShardId": "shardId-000000000000"
                    },
                    {
                        "ErrorCode": "InternalFailure",
                        "ErrorMessage": "Internal service failure."
                    }
                ]
            }"#,
            )
            .create_async()
            .await;

        let client = make_client(Some(server.url())).await;

        let records = vec![
            PutRecordsRequestEntry::builder()
                .data(Blob::new("test-data-1"))
                .partition_key("partition-1")
                .build()
                .unwrap(),
            PutRecordsRequestEntry::builder()
                .data(Blob::new("test-data-2"))
                .partition_key("partition-2")
                .build()
                .unwrap(),
        ];

        let result = add_records(&client, "test-stream", records).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.failed_record_count(), Some(1));
        assert_eq!(output.records().len(), 2);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_add_records_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/")
            .match_header("content-type", "application/x-amz-json-1.1")
            .match_header("x-amz-target", "Kinesis_20131202.PutRecords")
            .with_status(400)
            .with_body(
                r#"{
                "__type": "ResourceNotFoundException",
                "message": "Stream test-stream under account 123456789012 not found."
            }"#,
            )
            .create_async()
            .await;

        let client = make_client(Some(server.url())).await;

        let records = vec![
            PutRecordsRequestEntry::builder()
                .data(Blob::new("test-data"))
                .partition_key("partition")
                .build()
                .unwrap(),
        ];

        let result = add_records(&client, "test-stream", records).await;

        assert!(result.is_err());
        mock.assert_async().await;
    }
}
