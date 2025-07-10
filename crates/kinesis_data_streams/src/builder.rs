use crate::error::Error;
use aws_sdk_kinesis::primitives::Blob;
use aws_sdk_kinesis::types::PutRecordsRequestEntry;
use uuid::Uuid;

// 制限等の情報
// https://docs.aws.amazon.com/kinesis/latest/APIReference/API_PutRecords.html
pub struct RecordsBuilder {
    entries: Vec<PutRecordsRequestEntry>,
    total_size: usize,
    single_limit: usize, // 単一エントリのサイズ
    total_limit: usize,  // 合計サイズの制限
    record_limit: usize, // レコードの制限
}

impl RecordsBuilder {
    pub fn new() -> Self {
        Self::new_with_limit(1_000_000, 5_000_000, 500)
    }

    pub fn new_with_limit(single_limit: usize, total_limit: usize, record_limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            total_size: 0,
            single_limit,
            total_limit,
            record_limit,
        }
    }

    pub fn build(self) -> Vec<PutRecordsRequestEntry> {
        self.entries
    }

    pub fn add_entry_data(&mut self, data: impl Into<Vec<u8>>) -> Result<(), Error> {
        self.add_entry(data, None, None)
    }

    pub fn add_entry(
        &mut self,
        data: impl Into<Vec<u8>>,
        partition_key: Option<String>,
        explicit_hash_key: Option<String>,
    ) -> Result<(), Error> {
        // パーティションキーを指定しない場合は、UUIDを生成
        let partition_key = match partition_key {
            Some(key) => key,
            None => Uuid::now_v7().to_string(), // デフォルトのパーティションキーを生成
        };

        // 単体のサイズチェック
        let data: Vec<u8> = data.into();
        let size = data.len() + partition_key.len();
        if size >= self.single_limit {
            // 単体サイズを超える場合は追加しない
            return Err(Error::EntryOverItem(format!(
                "data size: {}, single_limit: {}",
                size, self.single_limit
            )));
        }

        // 合計サイズチェック
        if self.total_size + size >= self.total_limit || self.entries.len() >= self.record_limit {
            // 合計サイズを超える場合は追加しない
            return Err(Error::EntryOverAll(format!(
                "total size: {}, total_limit: {}, entries: {}, record_limit: {}",
                self.total_size + size,
                self.total_limit,
                self.entries.len() + 1,
                self.record_limit
            )));
        }
        let blob = Blob::new(data);
        let entry = PutRecordsRequestEntry::builder()
            .data(blob)
            .partition_key(partition_key)
            .set_explicit_hash_key(explicit_hash_key)
            .build()
            .map_err(Box::new)?;
        self.entries.push(entry);
        self.total_size += size;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for RecordsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RUST_LOG=debug REALM_CODE=test cargo test test_kinesis_data_streams_records -- --nocapture --test-threads=1
    #[tokio::test]
    async fn test_kinesis_data_streams_records() -> anyhow::Result<()> {
        let mut records = RecordsBuilder::new_with_limit(10, 20, 3);

        // 単体サイズオーバー
        match records.add_entry("0123456789".to_string(), Some("".to_string()), None) {
            Err(Error::EntryOverItem(_)) => {}
            _ => panic!("unexpected error"),
        }
        records.add_entry("012345678".to_string(), Some("".to_string()), None)?;
        records.add_entry("012345678".to_string(), Some("".to_string()), None)?;

        // 合計サイズオーバー
        match records.add_entry("012345678".to_string(), Some("".to_string()), None) {
            Err(Error::EntryOverAll(_)) => {}
            _ => panic!("unexpected error"),
        }
        records.add_entry("0".to_string(), Some("".to_string()), None)?;

        // レコード数オーバー
        match records.add_entry("0".to_string(), Some("".to_string()), None) {
            Err(Error::EntryOverAll(_)) => {}
            _ => panic!("unexpected error"),
        }
        Ok(())
    }
}
