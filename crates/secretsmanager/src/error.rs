use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] aws_sdk_secretsmanager::error::BuildError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_secretsmanager::Error>),

    #[error("ValidationError: {0}")]
    ValidationError(String),

    #[error("Secret not found")]
    NotFound,
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_secretsmanager::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
