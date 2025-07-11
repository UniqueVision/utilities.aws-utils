pub use crate::error::Error;
use std::collections::HashMap;

use aws_sdk_sqs::{
    Client,
    operation::{
        create_queue::CreateQueueOutput, delete_message::DeleteMessageOutput,
        delete_message_batch::DeleteMessageBatchOutput, delete_queue::DeleteQueueOutput,
        receive_message::ReceiveMessageOutput, send_message::SendMessageOutput,
        send_message_batch::SendMessageBatchOutput,
    },
    types::{
        DeleteMessageBatchRequestEntry, MessageAttributeValue, MessageSystemAttributeName,
        MessageSystemAttributeNameForSends, MessageSystemAttributeValue, QueueAttributeName,
        SendMessageBatchRequestEntry,
    },
};

use crate::error::from_aws_sdk_error;

pub async fn create_queue(
    client: &Client,
    queue_name: impl Into<String>,
    attributes: HashMap<QueueAttributeName, String>,
    tags: Option<HashMap<String, String>>,
) -> Result<CreateQueueOutput, Error> {
    client
        .create_queue()
        .set_queue_name(Some(queue_name.into()))
        .set_attributes(Some(attributes))
        .set_tags(tags)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn delete_queue(
    client: &Client,
    queue_url: impl Into<String>,
) -> Result<DeleteQueueOutput, Error> {
    client
        .delete_queue()
        .set_queue_url(Some(queue_url.into()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn receive_message(
    client: &Client,
    queue_url: impl Into<String>,
    max_number_of_messages: Option<i32>,
    message_attribute_names: Option<Vec<String>>,
    message_system_attribute_names: Option<Vec<MessageSystemAttributeName>>,
    receive_request_attempt_id: Option<String>,
    visibility_timeout: Option<i32>,
    wait_time_seconds: Option<i32>,
) -> Result<ReceiveMessageOutput, Error> {
    client
        .receive_message()
        .set_queue_url(Some(queue_url.into()))
        .set_max_number_of_messages(max_number_of_messages)
        .set_message_attribute_names(message_attribute_names)
        .set_message_system_attribute_names(message_system_attribute_names)
        .set_receive_request_attempt_id(receive_request_attempt_id)
        .set_visibility_timeout(visibility_timeout)
        .set_wait_time_seconds(wait_time_seconds)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn send_message(
    client: &Client,
    queue_url: impl Into<String>,
    message: Option<String>,
    message_group_id: Option<String>,
    message_deduplication_id: Option<String>,
    delay_seconds: Option<i32>,
    message_attributes: Option<HashMap<String, MessageAttributeValue>>,
    message_system_attributes: Option<
        HashMap<MessageSystemAttributeNameForSends, MessageSystemAttributeValue>,
    >,
) -> Result<SendMessageOutput, Error> {
    client
        .send_message()
        .set_queue_url(Some(queue_url.into()))
        .set_message_body(message)
        .set_message_group_id(message_group_id)
        .set_message_deduplication_id(message_deduplication_id)
        .set_delay_seconds(delay_seconds)
        .set_message_attributes(message_attributes)
        .set_message_system_attributes(message_system_attributes)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn send_message_batch(
    client: &Client,
    queue_url: impl Into<String>,
    entries: Vec<SendMessageBatchRequestEntry>,
) -> Result<SendMessageBatchOutput, Error> {
    client
        .send_message_batch()
        .set_queue_url(Some(queue_url.into()))
        .set_entries(Some(entries))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn delete_message(
    client: &Client,
    queue_url: impl Into<String>,
    handle: impl Into<String>,
) -> Result<DeleteMessageOutput, Error> {
    client
        .delete_message()
        .set_queue_url(Some(queue_url.into()))
        .set_receipt_handle(Some(handle.into()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn delete_message_batch(
    client: &Client,
    queue_url: impl Into<String>,
    entries: Vec<DeleteMessageBatchRequestEntry>,
) -> Result<DeleteMessageBatchOutput, Error> {
    client
        .delete_message_batch()
        .set_queue_url(Some(queue_url.into()))
        .set_entries(Some(entries))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}
