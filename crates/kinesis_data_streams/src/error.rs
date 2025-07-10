use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] Box<aws_sdk_kinesis::error::BuildError>),

    #[error("EntryOverAll {0}")]
    EntryOverAll(String),

    #[error("EntryOverItem {0}")]
    EntryOverItem(String),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_kinesis::Error>),
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_kinesis::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
