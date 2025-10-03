use aws_sdk_s3::{presigning::PresigningConfigError, primitives::ByteStreamError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BuildError(#[from] aws_sdk_s3::error::BuildError),

    #[error(transparent)]
    ByteStream(#[from] ByteStreamError),

    #[error(transparent)]
    Presigned(#[from] PresigningConfigError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_s3::Error>),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("ValidationError: {0}")]
    ValidationError(String),
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_s3::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}

impl Error {
    pub fn is_no_such_key(&self) -> bool {
        if let Error::AwsSdk(e) = self {
            matches!(**e, aws_sdk_s3::Error::NoSuchKey(_))
        } else {
            false
        }
    }

    pub fn is_no_such_bucket(&self) -> bool {
        if let Error::AwsSdk(e) = self {
            matches!(**e, aws_sdk_s3::Error::NoSuchBucket(_))
        } else {
            false
        }
    }

    pub fn is_not_found(&self) -> bool {
        if let Error::AwsSdk(e) = self {
            matches!(**e, aws_sdk_s3::Error::NotFound(_))
        } else {
            false
        }
    }
}
