use aws_sdk_sqs::types::{MessageAttributeValue, SendMessageBatchRequestEntry};
use std::collections::HashMap;

#[derive(Default)]
pub struct SendMessageBatchEntriesBuilder {
    entries: Vec<SendMessageBatchRequestEntry>,
}

impl SendMessageBatchEntriesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(mut self, id: impl Into<String>, message_body: impl Into<String>) -> Self {
        let entry = SendMessageBatchRequestEntry::builder()
            .id(id)
            .message_body(message_body)
            .build()
            .expect("id and message_body are required");
        self.entries.push(entry);
        self
    }

    pub fn add_message_with_delay(
        mut self,
        id: impl Into<String>,
        message_body: impl Into<String>,
        delay_seconds: i32,
    ) -> Self {
        let entry = SendMessageBatchRequestEntry::builder()
            .id(id)
            .message_body(message_body)
            .delay_seconds(delay_seconds)
            .build()
            .expect("id and message_body are required");
        self.entries.push(entry);
        self
    }

    pub fn add_fifo_message(
        mut self,
        id: impl Into<String>,
        message_body: impl Into<String>,
        message_group_id: impl Into<String>,
        message_deduplication_id: Option<String>,
    ) -> Self {
        let mut builder = SendMessageBatchRequestEntry::builder()
            .id(id)
            .message_body(message_body)
            .message_group_id(message_group_id);

        if let Some(dedup_id) = message_deduplication_id {
            builder = builder.message_deduplication_id(dedup_id);
        }

        let entry = builder.build().expect("id and message_body are required");
        self.entries.push(entry);
        self
    }

    pub fn add_message_with_attributes(
        mut self,
        id: impl Into<String>,
        message_body: impl Into<String>,
        attributes: HashMap<String, MessageAttributeValue>,
    ) -> Self {
        let entry = SendMessageBatchRequestEntry::builder()
            .id(id)
            .message_body(message_body)
            .set_message_attributes(Some(attributes))
            .build()
            .expect("id and message_body are required");
        self.entries.push(entry);
        self
    }

    pub fn add_entry(mut self, entry: SendMessageBatchRequestEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn build(self) -> Result<Vec<SendMessageBatchRequestEntry>, SendMessageBatchError> {
        if self.entries.is_empty() {
            return Err(SendMessageBatchError::EmptyBatch);
        }

        if self.entries.len() > 10 {
            return Err(SendMessageBatchError::TooManyMessages(self.entries.len()));
        }

        let mut seen_ids = std::collections::HashSet::new();
        for entry in &self.entries {
            if !seen_ids.insert(entry.id()) {
                return Err(SendMessageBatchError::DuplicateId(entry.id().to_string()));
            }
        }

        Ok(self.entries)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendMessageBatchError {
    #[error("Batch cannot be empty")]
    EmptyBatch,

    #[error("Batch contains {0} messages, maximum is 10")]
    TooManyMessages(usize),

    #[error("Duplicate message ID: {0}")]
    DuplicateId(String),
}

pub struct MessageEntryBuilder {
    id: String,
    message_body: String,
    delay_seconds: Option<i32>,
    message_attributes: Option<HashMap<String, MessageAttributeValue>>,
    message_group_id: Option<String>,
    message_deduplication_id: Option<String>,
}

impl MessageEntryBuilder {
    pub fn new(id: impl Into<String>, message_body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            message_body: message_body.into(),
            delay_seconds: None,
            message_attributes: None,
            message_group_id: None,
            message_deduplication_id: None,
        }
    }

    pub fn delay_seconds(mut self, seconds: i32) -> Self {
        self.delay_seconds = Some(seconds);
        self
    }

    pub fn add_attribute(mut self, key: impl Into<String>, value: MessageAttributeValue) -> Self {
        self.message_attributes
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }

    pub fn message_group_id(mut self, group_id: impl Into<String>) -> Self {
        self.message_group_id = Some(group_id.into());
        self
    }

    pub fn message_deduplication_id(mut self, dedup_id: impl Into<String>) -> Self {
        self.message_deduplication_id = Some(dedup_id.into());
        self
    }

    pub fn build(self) -> SendMessageBatchRequestEntry {
        let mut builder = SendMessageBatchRequestEntry::builder()
            .id(self.id)
            .message_body(self.message_body);

        if let Some(delay) = self.delay_seconds {
            builder = builder.delay_seconds(delay);
        }

        if let Some(attrs) = self.message_attributes {
            builder = builder.set_message_attributes(Some(attrs));
        }

        if let Some(group_id) = self.message_group_id {
            builder = builder.message_group_id(group_id);
        }

        if let Some(dedup_id) = self.message_deduplication_id {
            builder = builder.message_deduplication_id(dedup_id);
        }

        builder.build().expect("id and message_body are required")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_batch() {
        let batch = SendMessageBatchEntriesBuilder::new()
            .add_message("msg1", "Hello World")
            .add_message("msg2", "Goodbye World")
            .build()
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].id(), "msg1");
        assert_eq!(batch[0].message_body(), "Hello World");
    }

    #[test]
    fn test_message_with_delay() {
        let batch = SendMessageBatchEntriesBuilder::new()
            .add_message_with_delay("delayed", "Wait for me", 60)
            .build()
            .unwrap();

        assert_eq!(batch[0].delay_seconds(), Some(60));
    }

    #[test]
    fn test_fifo_message() {
        let batch = SendMessageBatchEntriesBuilder::new()
            .add_fifo_message(
                "fifo1",
                "Order matters",
                "group1",
                Some("dedup123".to_string()),
            )
            .build()
            .unwrap();

        assert_eq!(batch[0].message_group_id(), Some("group1"));
        assert_eq!(batch[0].message_deduplication_id(), Some("dedup123"));
    }

    #[test]
    fn test_too_many_messages() {
        let mut builder = SendMessageBatchEntriesBuilder::new();
        for i in 0..11 {
            builder = builder.add_message(format!("msg{i}"), "content");
        }

        match builder.build() {
            Err(SendMessageBatchError::TooManyMessages(11)) => {}
            _ => panic!("Expected TooManyMessages error"),
        }
    }

    #[test]
    fn test_duplicate_ids() {
        let result = SendMessageBatchEntriesBuilder::new()
            .add_message("same_id", "First")
            .add_message("same_id", "Second")
            .build();

        match result {
            Err(SendMessageBatchError::DuplicateId(id)) => assert_eq!(id, "same_id"),
            _ => panic!("Expected DuplicateId error"),
        }
    }

    #[test]
    fn test_empty_batch() {
        let result = SendMessageBatchEntriesBuilder::new().build();

        match result {
            Err(SendMessageBatchError::EmptyBatch) => {}
            _ => panic!("Expected EmptyBatch error"),
        }
    }

    #[test]
    fn test_message_entry_builder() {
        let attr = MessageAttributeValue::builder()
            .data_type("String")
            .string_value("test_value")
            .build()
            .unwrap();

        let entry = MessageEntryBuilder::new("custom", "Custom message")
            .delay_seconds(30)
            .add_attribute("key1", attr)
            .message_group_id("custom_group")
            .build();

        assert_eq!(entry.id(), "custom");
        assert_eq!(entry.delay_seconds(), Some(30));
        assert!(entry.message_attributes().is_some());
        assert_eq!(entry.message_group_id(), Some("custom_group"));
    }
}
