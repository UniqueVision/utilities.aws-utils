use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] aws_sdk_sqs::error::BuildError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_sqs::Error>),
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_sqs::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
