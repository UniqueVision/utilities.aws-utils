use aws_sdk_sqs::error::SdkError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("見つかりません")]
    NotFound,

    #[error("CreateQueueError {0}")]
    CreateQueueError(#[from] SdkError<aws_sdk_sqs::operation::create_queue::CreateQueueError>),

    #[error("DeleteQueueError {0}")]
    DeleteQueueError(#[from] SdkError<aws_sdk_sqs::operation::delete_queue::DeleteQueueError>),

    #[error("ReceiveMessageError {0}")]
    ReceiveMessageError(
        #[from] SdkError<aws_sdk_sqs::operation::receive_message::ReceiveMessageError>,
    ),

    #[error("SendMessageError {0}")]
    SendMessageError(#[from] SdkError<aws_sdk_sqs::operation::send_message::SendMessageError>),

    #[error("SendMessageBatchError {0}")]
    SendMessageBatchError(
        #[from] SdkError<aws_sdk_sqs::operation::send_message_batch::SendMessageBatchError>,
    ),

    #[error("DeleteMessageError {0}")]
    DeleteMessageError(
        #[from] SdkError<aws_sdk_sqs::operation::delete_message::DeleteMessageError>,
    ),

    #[error("DeleteMessageBatchError {0}")]
    DeleteMessageBatchError(
        #[from] SdkError<aws_sdk_sqs::operation::delete_message_batch::DeleteMessageBatchError>,
    ),

    #[error(transparent)]
    BuildError(#[from] aws_sdk_sqs::error::BuildError),

    #[error(transparent)]
    AwsSdk(#[from] Box<aws_sdk_sqs::Error>),
}

pub(crate) fn from_aws_sdk_error(e: impl Into<aws_sdk_sqs::Error>) -> Error {
    Error::AwsSdk(Box::new(e.into()))
}
