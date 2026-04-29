use std::time::Duration;

use redis::aio::ConnectionManager;

pub struct MlCache {
    redis: ConnectionManager,
    ttl: Duration,
}

impl MlCache {
    pub fn new(redis: ConnectionManager) -> Self {
        Self {
            redis,
            ttl: Duration::from_secs(3600),
        }
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        model: &str,
    ) -> Option<T> {
        let key = format!("ml:{model}:{}", chrono::Utc::now().format("%Y%m%d%H"));
        let mut conn = self.redis.clone();
        let val: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .ok()
            .flatten();
        val.and_then(|v| serde_json::from_str(&v).ok())
    }

    pub async fn set<T: serde::Serialize>(
        &self,
        model: &str,
        data: &T,
    ) {
        let key = format!("ml:{model}:{}", chrono::Utc::now().format("%Y%m%d%H"));
        let mut conn = self.redis.clone();
        if let Ok(val) = serde_json::to_string(data) {
            let _: Result<(), _> = redis::cmd("SETEX")
                .arg(&key)
                .arg(self.ttl.as_secs() as i64)
                .arg(&val)
                .query_async(&mut conn)
                .await;
        }
    }
}
