pub use crate::error::Error;
use std::collections::HashMap;

use aws_sdk_sqs::{
    operation::{
        create_queue::CreateQueueOutput, delete_queue::DeleteQueueOutput,
        receive_message::ReceiveMessageOutput, send_message::SendMessageOutput,
        send_message_batch::SendMessageBatchOutput,
    }, types::{
        builders::{DeleteMessageBatchRequestEntryBuilder, SendMessageBatchRequestEntryBuilder}, MessageAttributeValue, MessageSystemAttributeName, MessageSystemAttributeNameForSends, MessageSystemAttributeValue, QueueAttributeName, SendMessageBatchRequestEntry
    }, Client
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

#[derive(Debug, Clone)]
pub struct Sqs {
    client: Client,
    queue_url: String,
}

#[derive(Debug, Clone)]
pub struct SqsMessage {
    pub receipt_handle: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct SendMessageType {
    pub key: String,
    pub message: String,
}

impl Sqs {
    pub async fn new(queue_url: &str) -> Self {
        let config = aws_config::load_from_env().await;
        let mut builder = aws_sdk_sqs::config::Builder::from(&config);

        if let Ok(aws_endpoint_url) = std::env::var("AWS_ENDPOINT_URL_SQS") {
            builder = builder.endpoint_url(aws_endpoint_url)
        }
        let client = Client::from_conf(builder.build());
        Self {
            client,
            queue_url: queue_url.to_owned(),
        }
    }

    pub async fn create_queue(&self, queue_name: &str) -> Result<Option<String>, Error> {
        let mut attribute = HashMap::new();
        // 14日
        attribute.insert(
            QueueAttributeName::MessageRetentionPeriod,
            "1209600".to_owned(),
        );

        // 12時間
        attribute.insert(QueueAttributeName::VisibilityTimeout, "43200".to_owned());
        attribute.insert(QueueAttributeName::FifoQueue, "true".to_owned());
        attribute.insert(
            QueueAttributeName::DeduplicationScope,
            "messageGroup".to_owned(),
        );
        attribute.insert(
            QueueAttributeName::FifoThroughputLimit,
            "perMessageGroupId".to_owned(),
        );

        let resp = self
            .client
            .create_queue()
            .set_queue_name(Some(queue_name.to_owned()))
            .set_attributes(Some(attribute))
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
        Ok(resp.queue_url().map(|s| s.to_owned()))
    }

    pub async fn delete_queue(&self) -> Result<(), Error> {
        let _resp = self
            .client
            .delete_queue()
            .set_queue_url(Some(self.queue_url.clone()))
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
        Ok(())
    }

    pub async fn receive_message(
        &self,
        max_number_of_messages: Option<i32>,
    ) -> Result<Vec<SqsMessage>, Error> {
        let mut builder = self
            .client
            .receive_message()
            .set_queue_url(Some(self.queue_url.clone()));

        if let Some(max_number_of_messages) = max_number_of_messages {
            builder = builder.max_number_of_messages(max_number_of_messages);
        }

        let resp = builder.send().await.map_err(from_aws_sdk_error)?;
        let mut result = vec![];
        for message in resp.messages() {
            if let Some(body) = message.body() {
                result.push(SqsMessage {
                    receipt_handle: message.receipt_handle().unwrap_or_default().to_owned(),
                    message: body.to_owned(),
                });
            }
        }
        Ok(result)
    }

    pub async fn send_message(&self, message: SendMessageType) -> Result<(), Error> {
        let _resp = self
            .client
            .send_message()
            .set_queue_url(Some(self.queue_url.clone()))
            .set_message_body(Some(message.message))
            .set_message_group_id(Some(message.key.clone()))
            .set_message_deduplication_id(Some(message.key))
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
        Ok(())
    }

    pub async fn send_message_batch(&self, messages: &[SendMessageType]) -> Result<(), Error> {
        let mut entries = vec![];
        for (index, message) in messages.iter().enumerate() {
            let entry = SendMessageBatchRequestEntryBuilder::default()
                .id(format!("message_{index}"))
                .message_body(message.message.clone())
                .set_message_group_id(Some(message.key.clone()))
                .set_message_deduplication_id(Some(message.key.clone()))
                .build()?;
            entries.push(entry);
        }
        let _resp = self
            .client
            .send_message_batch()
            .set_queue_url(Some(self.queue_url.clone()))
            .set_entries(Some(entries))
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
        Ok(())
    }

    pub async fn delete_message(&self, handle: &str) -> Result<(), Error> {
        let _resp = self
            .client
            .delete_message()
            .set_queue_url(Some(self.queue_url.clone()))
            .set_receipt_handle(Some(handle.to_owned()))
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
        Ok(())
    }

    pub async fn delete_message_batch(&self, handles: &[String]) -> Result<(), Error> {
        let mut entries = vec![];
        for (index, handle) in handles.iter().enumerate() {
            let entry = DeleteMessageBatchRequestEntryBuilder::default()
                .id(format!("message_{index}"))
                .receipt_handle(handle.to_owned())
                .build()?;
            entries.push(entry);
        }

        let _resp = self
            .client
            .delete_message_batch()
            .set_queue_url(Some(self.queue_url.clone()))
            .set_entries(Some(entries))
            .send()
            .await
            .map_err(from_aws_sdk_error)?;
        Ok(())
    }

    pub fn queue_url(&self) -> &str {
        &self.queue_url
    }
}

#[cfg(test)]
mod tests {
    use crate::sqs::Sqs;

    // REALM_CODE=test cargo test -p sqs test_sqs_create_queue -- --nocapture --test-threads=1
    #[tokio::test]
    async fn test_sqs_create_queue() -> anyhow::Result<()> {
        let sqs_queue_url = std::env::var("SQS_QUEUE_URL")
            .unwrap_or_else(|_| "http://localhost:9324/000000000000.fifo".to_string());
        let sqs = Sqs::new(&sqs_queue_url).await;
        let _ = sqs.create_queue("000000000000.fifo").await?;

        Ok(())
    }
}
