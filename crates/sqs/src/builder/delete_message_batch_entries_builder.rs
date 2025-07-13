use aws_sdk_sqs::types::DeleteMessageBatchRequestEntry;
use std::collections::HashSet;

#[derive(Default)]
pub struct DeleteMessageBatchEntriesBuilder {
    entries: Vec<DeleteMessageBatchRequestEntry>,
}

impl DeleteMessageBatchEntriesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(mut self, id: impl Into<String>, receipt_handle: impl Into<String>) -> Self {
        let entry = DeleteMessageBatchRequestEntry::builder()
            .id(id)
            .receipt_handle(receipt_handle)
            .build()
            .expect("id and receipt_handle are required");
        self.entries.push(entry);
        self
    }

    pub fn add_entry(mut self, entry: DeleteMessageBatchRequestEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn build(self) -> Result<Vec<DeleteMessageBatchRequestEntry>, DeleteMessageBatchError> {
        if self.entries.is_empty() {
            return Err(DeleteMessageBatchError::EmptyBatch);
        }

        if self.entries.len() > 10 {
            return Err(DeleteMessageBatchError::TooManyMessages(self.entries.len()));
        }

        let mut seen_ids = HashSet::new();
        for entry in &self.entries {
            if !seen_ids.insert(entry.id()) {
                return Err(DeleteMessageBatchError::DuplicateId(entry.id().to_string()));
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
pub enum DeleteMessageBatchError {
    #[error("Batch cannot be empty")]
    EmptyBatch,

    #[error("Batch contains {0} messages, maximum is 10")]
    TooManyMessages(usize),

    #[error("Duplicate message ID: {0}")]
    DuplicateId(String),
}

pub struct DeleteMessageEntryBuilder {
    id: String,
    receipt_handle: String,
}

impl DeleteMessageEntryBuilder {
    pub fn new(id: impl Into<String>, receipt_handle: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            receipt_handle: receipt_handle.into(),
        }
    }

    pub fn build(self) -> DeleteMessageBatchRequestEntry {
        DeleteMessageBatchRequestEntry::builder()
            .id(self.id)
            .receipt_handle(self.receipt_handle)
            .build()
            .expect("id and receipt_handle are required")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_batch() {
        let batch = DeleteMessageBatchEntriesBuilder::new()
            .add_message("msg1", "receipt_handle_1")
            .add_message("msg2", "receipt_handle_2")
            .build()
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].id(), "msg1");
        assert_eq!(batch[0].receipt_handle(), "receipt_handle_1");
    }

    #[test]
    fn test_too_many_messages() {
        let mut builder = DeleteMessageBatchEntriesBuilder::new();
        for i in 0..11 {
            builder = builder.add_message(format!("msg{i}"), format!("receipt_{i}"));
        }

        match builder.build() {
            Err(DeleteMessageBatchError::TooManyMessages(11)) => {}
            _ => panic!("Expected TooManyMessages error"),
        }
    }

    #[test]
    fn test_duplicate_ids() {
        let result = DeleteMessageBatchEntriesBuilder::new()
            .add_message("same_id", "receipt_1")
            .add_message("same_id", "receipt_2")
            .build();

        match result {
            Err(DeleteMessageBatchError::DuplicateId(id)) => assert_eq!(id, "same_id"),
            _ => panic!("Expected DuplicateId error"),
        }
    }

    #[test]
    fn test_empty_batch() {
        let result = DeleteMessageBatchEntriesBuilder::new().build();

        match result {
            Err(DeleteMessageBatchError::EmptyBatch) => {}
            _ => panic!("Expected EmptyBatch error"),
        }
    }

    #[test]
    fn test_delete_entry_builder() {
        let entry = DeleteMessageEntryBuilder::new("custom", "custom_receipt").build();

        assert_eq!(entry.id(), "custom");
        assert_eq!(entry.receipt_handle(), "custom_receipt");
    }

    #[test]
    fn test_add_entry() {
        let entry = DeleteMessageBatchRequestEntry::builder()
            .id("direct")
            .receipt_handle("direct_receipt")
            .build()
            .unwrap();

        let batch = DeleteMessageBatchEntriesBuilder::new()
            .add_entry(entry)
            .build()
            .unwrap();

        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].id(), "direct");
        assert_eq!(batch[0].receipt_handle(), "direct_receipt");
    }
}
