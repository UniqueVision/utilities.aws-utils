use aws_sdk_sqs::types::QueueAttributeName;
use serde_json;

use crate::sqs::Error;

pub enum FifoThroughputLimit {
    PerMessageGroupId,
    PerQueue,
}

pub enum DeduplicationScope {
    MessageGroup,
    Queue,
}

#[derive(Debug, Clone)]
pub struct RedrivePolicy {
    pub max_receive_count: u32,
    pub dead_letter_target_arn: String,
}

impl RedrivePolicy {
    pub fn new(max_receive_count: u32, dead_letter_target_arn: String) -> Self {
        Self {
            max_receive_count,
            dead_letter_target_arn,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let policy = serde_json::json!({
            "maxReceiveCount": self.max_receive_count.to_string(),
            "deadLetterTargetArn": self.dead_letter_target_arn
        });
        serde_json::to_string(&policy)
    }
}

#[derive(Debug, Clone)]
pub enum RedrivePermission {
    AllowAll,
    DenyAll,
    ByQueue,
}

impl RedrivePermission {
    fn as_str(&self) -> &str {
        match self {
            RedrivePermission::AllowAll => "allowAll",
            RedrivePermission::DenyAll => "denyAll",
            RedrivePermission::ByQueue => "byQueue",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedriveAllowPolicy {
    pub redrive_permission: RedrivePermission,
    pub source_queue_arns: Option<Vec<String>>,
}

impl RedriveAllowPolicy {
    pub fn allow_all() -> Self {
        Self {
            redrive_permission: RedrivePermission::AllowAll,
            source_queue_arns: None,
        }
    }

    pub fn deny_all() -> Self {
        Self {
            redrive_permission: RedrivePermission::DenyAll,
            source_queue_arns: None,
        }
    }

    pub fn by_queue(source_queue_arns: Vec<String>) -> Self {
        Self {
            redrive_permission: RedrivePermission::ByQueue,
            source_queue_arns: Some(source_queue_arns),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let mut policy = serde_json::json!({
            "redrivePermission": self.redrive_permission.as_str()
        });

        if let Some(arns) = &self.source_queue_arns {
            policy["sourceQueueArns"] = serde_json::json!(arns);
        }

        serde_json::to_string(&policy)
    }
}

pub struct CreateQueueAttributeBuilder {
    delay_seconds: Option<u32>,
    maximum_message_size: Option<u32>,
    message_retention_period: Option<u32>,
    policy: Option<String>,
    receive_message_wait_time_seconds: Option<u32>,
    visibility_timeout: Option<u32>,
    redrive_policy: Option<RedrivePolicy>,
    redrive_allow_policy: Option<RedriveAllowPolicy>,
    content_based_deduplication: Option<bool>,
    kms_master_key_id: Option<String>,
    kms_data_key_reuse_period_seconds: Option<u32>,
    sqs_managed_sse_enabled: Option<bool>,
    fifo_throughput_limit: Option<FifoThroughputLimit>,
    deduplication_scope: Option<DeduplicationScope>,
}

impl Default for CreateQueueAttributeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateQueueAttributeBuilder {
    pub fn new() -> Self {
        Self {
            delay_seconds: None,
            maximum_message_size: None,
            message_retention_period: None,
            policy: None,
            receive_message_wait_time_seconds: None,
            visibility_timeout: None,
            redrive_policy: None,
            redrive_allow_policy: None,
            content_based_deduplication: None,
            kms_master_key_id: None,
            kms_data_key_reuse_period_seconds: None,
            sqs_managed_sse_enabled: None,
            fifo_throughput_limit: None,
            deduplication_scope: None,
        }
    }

    pub fn delay_seconds(mut self, value: u32) -> Self {
        self.delay_seconds = Some(value);
        self
    }

    pub fn maximum_message_size(mut self, value: u32) -> Self {
        self.maximum_message_size = Some(value);
        self
    }

    pub fn message_retention_period(mut self, value: u32) -> Self {
        self.message_retention_period = Some(value);
        self
    }

    pub fn policy(mut self, value: String) -> Self {
        self.policy = Some(value);
        self
    }

    pub fn receive_message_wait_time_seconds(mut self, value: u32) -> Self {
        self.receive_message_wait_time_seconds = Some(value);
        self
    }

    pub fn visibility_timeout(mut self, value: u32) -> Self {
        self.visibility_timeout = Some(value);
        self
    }

    pub fn redrive_policy(mut self, value: RedrivePolicy) -> Self {
        self.redrive_policy = Some(value);
        self
    }

    pub fn redrive_allow_policy(mut self, value: RedriveAllowPolicy) -> Self {
        self.redrive_allow_policy = Some(value);
        self
    }

    pub fn content_based_deduplication(mut self, value: bool) -> Self {
        self.content_based_deduplication = Some(value);
        self
    }

    pub fn kms_master_key_id(mut self, value: String) -> Self {
        self.kms_master_key_id = Some(value);
        self
    }

    pub fn kms_data_key_reuse_period_seconds(mut self, value: u32) -> Self {
        self.kms_data_key_reuse_period_seconds = Some(value);
        self
    }

    pub fn sqs_managed_sse_enabled(mut self, value: bool) -> Self {
        self.sqs_managed_sse_enabled = Some(value);
        self
    }

    pub fn fifo_throughput_limit(mut self, value: FifoThroughputLimit) -> Self {
        self.fifo_throughput_limit = Some(value);
        self
    }

    pub fn deduplication_scope(mut self, value: DeduplicationScope) -> Self {
        self.deduplication_scope = Some(value);
        self
    }

    pub fn build(self) -> Result<std::collections::HashMap<QueueAttributeName, String>, Error> {
        // Validate all attributes
        if let Some(value) = self.delay_seconds {
            if value > 900 {
                return Err(Error::ValidationError(
                    "DelaySeconds must be between 0 and 900 seconds.".to_string(),
                ));
            }
        }

        if let Some(value) = self.maximum_message_size {
            if !(1024..=262144).contains(&value) {
                return Err(Error::ValidationError(
                    "MaximumMessageSize must be between 1024 and 262144 bytes.".to_string(),
                ));
            }
        }

        if let Some(value) = self.message_retention_period {
            if !(60..=1209600).contains(&value) {
                return Err(Error::ValidationError(
                    "MessageRetentionPeriod must be between 60 and 1209600 seconds.".to_string(),
                ));
            }
        }

        if let Some(value) = self.receive_message_wait_time_seconds {
            if value > 20 {
                return Err(Error::ValidationError(
                    "ReceiveMessageWaitTimeSeconds must be between 0 and 20 seconds.".to_string(),
                ));
            }
        }

        if let Some(value) = self.visibility_timeout {
            if value > 43200 {
                return Err(Error::ValidationError(
                    "VisibilityTimeout must be between 0 and 43200 seconds.".to_string(),
                ));
            }
        }

        if let Some(value) = self.kms_data_key_reuse_period_seconds {
            if !(60..=86400).contains(&value) {
                return Err(Error::ValidationError(
                    "KmsDataKeyReusePeriodSeconds must be between 60 and 86400 seconds."
                        .to_string(),
                ));
            }
        }

        // Validate RedrivePolicy
        if let Some(ref redrive_policy) = self.redrive_policy {
            if !(1..=1000).contains(&redrive_policy.max_receive_count) {
                return Err(Error::ValidationError(
                    "maxReceiveCount must be between 1 and 1000.".to_string(),
                ));
            }
        }

        // Validate RedriveAllowPolicy
        if let Some(ref redrive_allow_policy) = self.redrive_allow_policy {
            if matches!(
                redrive_allow_policy.redrive_permission,
                RedrivePermission::ByQueue
            ) {
                if let Some(ref arns) = redrive_allow_policy.source_queue_arns {
                    if arns.is_empty() {
                        return Err(Error::ValidationError(
                            "sourceQueueArns cannot be empty when using byQueue permission."
                                .to_string(),
                        ));
                    }
                    if arns.len() > 10 {
                        return Err(Error::ValidationError(
                            "sourceQueueArns cannot contain more than 10 queue ARNs.".to_string(),
                        ));
                    }
                } else {
                    return Err(Error::ValidationError(
                        "sourceQueueArns must be provided when using byQueue permission."
                            .to_string(),
                    ));
                }
            }
        }

        // Validate FifoThroughputLimit and DeduplicationScope combination
        if let Some(ref fifo_limit) = self.fifo_throughput_limit {
            if matches!(fifo_limit, FifoThroughputLimit::PerMessageGroupId) {
                match self.deduplication_scope {
                    Some(DeduplicationScope::MessageGroup) => {
                        // Valid combination
                    }
                    _ => {
                        return Err(Error::ValidationError(
                            "FifoThroughputLimit.PerMessageGroupId requires DeduplicationScope.MessageGroup".to_string()
                        ));
                    }
                }
            }
        }

        let mut attributes = std::collections::HashMap::new();

        if let Some(value) = self.delay_seconds {
            attributes.insert(QueueAttributeName::DelaySeconds, value.to_string());
        }
        if let Some(value) = self.maximum_message_size {
            attributes.insert(QueueAttributeName::MaximumMessageSize, value.to_string());
        }
        if let Some(value) = self.message_retention_period {
            attributes.insert(
                QueueAttributeName::MessageRetentionPeriod,
                value.to_string(),
            );
        }
        if let Some(value) = self.policy {
            attributes.insert(QueueAttributeName::Policy, value);
        }
        if let Some(value) = self.receive_message_wait_time_seconds {
            attributes.insert(
                QueueAttributeName::ReceiveMessageWaitTimeSeconds,
                value.to_string(),
            );
        }
        if let Some(value) = self.visibility_timeout {
            attributes.insert(QueueAttributeName::VisibilityTimeout, value.to_string());
        }
        if let Some(value) = self.redrive_policy {
            if let Ok(json) = value.to_json() {
                attributes.insert(QueueAttributeName::RedrivePolicy, json);
            }
        }
        if let Some(value) = self.redrive_allow_policy {
            if let Ok(json) = value.to_json() {
                attributes.insert(QueueAttributeName::RedriveAllowPolicy, json);
            }
        }
        if let Some(value) = self.content_based_deduplication {
            attributes.insert(
                QueueAttributeName::ContentBasedDeduplication,
                value.to_string(),
            );
        }
        if let Some(value) = self.kms_master_key_id {
            attributes.insert(QueueAttributeName::KmsMasterKeyId, value);
        }
        if let Some(value) = self.kms_data_key_reuse_period_seconds {
            attributes.insert(
                QueueAttributeName::KmsDataKeyReusePeriodSeconds,
                value.to_string(),
            );
        }
        if let Some(value) = self.sqs_managed_sse_enabled {
            attributes.insert(QueueAttributeName::SqsManagedSseEnabled, value.to_string());
        }
        if let Some(value) = self.fifo_throughput_limit {
            let value_str = match value {
                FifoThroughputLimit::PerMessageGroupId => "perMessageGroupId",
                FifoThroughputLimit::PerQueue => "perQueue",
            };
            attributes.insert(
                QueueAttributeName::FifoThroughputLimit,
                value_str.to_string(),
            );
        }
        if let Some(value) = self.deduplication_scope {
            let value_str = match value {
                DeduplicationScope::MessageGroup => "messageGroup",
                DeduplicationScope::Queue => "queue",
            };
            attributes.insert(
                QueueAttributeName::DeduplicationScope,
                value_str.to_string(),
            );
        }

        Ok(attributes)
    }
}
