use std::time::Duration;

use redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct MlCache {
    redis: ConnectionManager,
    default_ttl: Duration,
}

impl MlCache {
    pub fn new(redis: ConnectionManager) -> Self {
        Self {
            redis,
            default_ttl: Duration::from_secs(3600),
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Option<T> {
        let cache_key = format!("ml:{key}");
        let mut conn = self.redis.clone();
        let val: Option<String> = redis::cmd("GET")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await
            .ok()
            .flatten();
        val.and_then(|v| serde_json::from_str(&v).ok())
    }

    pub async fn set<T: serde::Serialize>(
        &self,
        key: &str,
        data: &T,
    ) {
        self.set_with_ttl(key, data, self.default_ttl).await;
    }

    pub async fn set_with_ttl<T: serde::Serialize>(
        &self,
        key: &str,
        data: &T,
        ttl: Duration,
    ) {
        let cache_key = format!("ml:{key}");
        let mut conn = self.redis.clone();
        if let Ok(val) = serde_json::to_string(data) {
            let _: Result<(), _> = redis::cmd("SETEX")
                .arg(&cache_key)
                .arg(ttl.as_secs() as i64)
                .arg(&val)
                .query_async(&mut conn)
                .await;
        }
    }

    pub async fn invalidate(&self, key: &str) {
        let cache_key = format!("ml:{key}");
        let mut conn = self.redis.clone();
        let _: Result<(), _> = redis::cmd("DEL")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await;
    }
}
