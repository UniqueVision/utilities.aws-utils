use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("PutItemError {0}")]
    KinesisPutItemError(#[from] Box<aws_sdk_kinesis::operation::put_record::PutRecordError>),

    #[error("PutRecordsError {0}")]
    KinesisPutRecordsError(#[from] Box<aws_sdk_kinesis::operation::put_records::PutRecordsError>),

    #[error("BuildError {0}")]
    KinesisBuildError(#[from] Box<aws_sdk_kinesis::error::BuildError>),

    #[error("EntryOverAll")]
    EntryOverAll(String),

    #[error("EntryOverItem")]
    EntryOverItem(String),
}
