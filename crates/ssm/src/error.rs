use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] aws_sdk_ssm::error::BuildError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_ssm::Error>),

    #[error("ValidationError: {0}")]
    ValidationError(String),

    #[error("Parameter not found")]
    NotFound,
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_ssm::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
