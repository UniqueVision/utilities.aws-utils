use std::{collections::HashMap, hash::Hash, time::Duration, future::Future};

use aws_sdk_dynamodb::Client;
use chrono::prelude::*;

use crate::error::Error;


pub struct HolderMap<K, V> {
    map: HashMap<K, (V, DateTime<Utc>)>,
    client: Client,
    expiration: Duration,
}

impl<K,V> HolderMap<K, V> 
where
    K: PartialEq + Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(client: Client, expiration: Duration) -> Self {
        HolderMap {
            map: HashMap::new(),
            client,
            expiration,
        }
    }

    pub async fn get<FutOne>(&mut self, key: &K, f: impl FnOnce(Client, K) -> FutOne, now: Option<DateTime<Utc>>, ) -> Result<Option<V>, Error>
     where
        FutOne: Future<Output = Result<Option<V>, Error>>,
    {
        match self.map.get(key) {
            Some((value, expire_at)) if get_now(now) < *expire_at => {
                return Ok(Some(value.clone()));
            }
            _ => {}
        }
        let client = self.client.clone();
        let Some(value) = f(client, key.clone()).await? else {
            return Ok(None);
        };
        self.map.insert(
            key.clone(),
            (value.clone(), expire_at(now, self.expiration)),
        );
        Ok(Some(value))
    }
}

fn get_now(now: Option<DateTime<Utc>>) -> DateTime<Utc> {
    now.unwrap_or(Utc::now())
}

fn expire_at(now: Option<DateTime<Utc>>, interval: Duration) -> DateTime<Utc> {
    get_now(now) + interval
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone, Debug, PartialEq)]
    struct TestValue(String);

    async fn create_test_client() -> Client {
        let mut server = mockito::Server::new_async().await;
        let mock_url = server.url();
        
        // Create a mock endpoint that won't be called
        let _mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body("{}")
            .create_async()
            .await;
        
        crate::make_client(Some(mock_url)).await
    }

    #[tokio::test]
    async fn test_holder_map_new() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        assert_eq!(holder.map.len(), 0);
        assert_eq!(holder.expiration, expiration);
    }

    #[tokio::test]
    async fn test_holder_map_get_cache_miss() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let mut holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        let key = "test_key".to_string();
        let expected_value = TestValue("test_value".to_string());
        let expected_clone = expected_value.clone();
        
        let result = holder.get(
            &key,
            |_client, _key| async move {
                Ok(Some(expected_clone))
            },
            None,
        ).await.unwrap();
        
        assert_eq!(result, Some(expected_value.clone()));
        assert_eq!(holder.map.len(), 1);
        assert!(holder.map.contains_key(&key));
    }

    #[tokio::test]
    async fn test_holder_map_get_cache_hit() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let mut holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        let key = "test_key".to_string();
        let expected_value = TestValue("test_value".to_string());
        let now = Utc::now();
        
        // First call to populate cache
        let expected_clone = expected_value.clone();
        let result1 = holder.get(
            &key,
            |_client, _key| async move {
                Ok(Some(expected_clone))
            },
            Some(now),
        ).await.unwrap();
        
        assert_eq!(result1, Some(expected_value.clone()));
        
        // Second call should hit cache
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();
        
        let result2 = holder.get(
            &key,
            |_client, _key| {
                let call_count = call_count_clone.clone();
                async move {
                    let mut count = call_count.lock().await;
                    *count += 1;
                    Ok(Some(TestValue("should_not_be_returned".to_string())))
                }
            },
            Some(now + chrono::Duration::seconds(30)),
        ).await.unwrap();
        
        assert_eq!(result2, Some(expected_value));
        assert_eq!(*call_count.lock().await, 0); // Function should not be called
    }

    #[tokio::test]
    async fn test_holder_map_get_cache_expired() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let mut holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        let key = "test_key".to_string();
        let old_value = TestValue("old_value".to_string());
        let new_value = TestValue("new_value".to_string());
        let now = Utc::now();
        
        // First call to populate cache
        let old_value_clone = old_value.clone();
        let result1 = holder.get(
            &key,
            |_client, _key| async move {
                Ok(Some(old_value_clone))
            },
            Some(now),
        ).await.unwrap();
        
        assert_eq!(result1, Some(old_value));
        
        // Second call with expired cache
        let new_value_clone = new_value.clone();
        let result2 = holder.get(
            &key,
            |_client, _key| async move {
                Ok(Some(new_value_clone))
            },
            Some(now + chrono::Duration::seconds(61)),
        ).await.unwrap();
        
        assert_eq!(result2, Some(new_value));
    }

    #[tokio::test]
    async fn test_holder_map_get_none_value() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let mut holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        let key = "test_key".to_string();
        
        let result = holder.get(
            &key,
            |_client, _key| async move {
                Ok(None)
            },
            None,
        ).await.unwrap();
        
        assert_eq!(result, None);
        assert_eq!(holder.map.len(), 0); // Nothing should be cached
    }

    #[tokio::test]
    async fn test_holder_map_get_error() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let mut holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        let key = "test_key".to_string();
        
        let result = holder.get(
            &key,
            |_client, _key| async move {
                Err(Error::Invalid("Test error".to_string()))
            },
            None,
        ).await;
        
        assert!(result.is_err());
        assert_eq!(holder.map.len(), 0); // Nothing should be cached on error
    }

    #[tokio::test]
    async fn test_holder_map_multiple_keys() {
        let client = create_test_client().await;
        let expiration = Duration::from_secs(60);
        let mut holder: HolderMap<String, TestValue> = HolderMap::new(client, expiration);
        
        let key1 = "key1".to_string();
        let key2 = "key2".to_string();
        let value1 = TestValue("value1".to_string());
        let value2 = TestValue("value2".to_string());
        
        // Add first key
        let value1_clone = value1.clone();
        let result1 = holder.get(
            &key1,
            |_client, _key| async move {
                Ok(Some(value1_clone))
            },
            None,
        ).await.unwrap();
        
        assert_eq!(result1, Some(value1.clone()));
        
        // Add second key
        let value2_clone = value2.clone();
        let result2 = holder.get(
            &key2,
            |_client, _key| async move {
                Ok(Some(value2_clone))
            },
            None,
        ).await.unwrap();
        
        assert_eq!(result2, Some(value2.clone()));
        assert_eq!(holder.map.len(), 2);
        
        // Verify both keys are cached
        assert!(holder.map.contains_key(&key1));
        assert!(holder.map.contains_key(&key2));
    }

    #[test]
    fn test_get_now_with_none() {
        let now = get_now(None);
        let expected = Utc::now();
        
        // Allow 1 second difference due to execution time
        assert!((now - expected).num_seconds().abs() <= 1);
    }

    #[test]
    fn test_get_now_with_some() {
        let specific_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let now = get_now(Some(specific_time));
        
        assert_eq!(now, specific_time);
    }

    #[test]
    fn test_expire_at() {
        let specific_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let interval = Duration::from_secs(3600); // 1 hour
        let expire_time = expire_at(Some(specific_time), interval);
        
        let expected = Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap();
        assert_eq!(expire_time, expected);
    }

    #[test]
    fn test_expire_at_with_none() {
        let interval = Duration::from_secs(60);
        let expire_time = expire_at(None, interval);
        let expected = Utc::now() + chrono::Duration::seconds(60);
        
        // Allow 1 second difference due to execution time
        assert!((expire_time - expected).num_seconds().abs() <= 1);
    }
}