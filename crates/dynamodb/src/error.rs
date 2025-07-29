use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] aws_sdk_dynamodb::error::BuildError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_dynamodb::Error>),

    #[error("ValidationError: {0}")]
    ValidationError(String),

    #[error("NotFound")]
    NotFound,

    #[error("Invalid: {0}")]
    Invalid(String),
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_dynamodb::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
