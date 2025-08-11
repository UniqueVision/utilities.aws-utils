use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] aws_sdk_firehose::error::BuildError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_firehose::Error>),

    #[error("Invalid: {0}")]
    Invalid(String),
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_firehose::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
