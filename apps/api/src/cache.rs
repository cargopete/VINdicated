use anyhow::Result;
use redis::AsyncCommands;
use std::time::Duration;

pub struct Cache {
    pool: redis::aio::MultiplexedConnection,
}

impl Cache {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { pool: conn })
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.pool.clone();
        conn.get::<_, Option<String>>(key).await.ok().flatten()
    }

    pub async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<()> {
        let mut conn = self.pool.clone();
        conn.set_ex::<_, _, ()>(key, value, ttl.as_secs()).await?;
        Ok(())
    }

    pub fn vin_key(source_id: &str, vin: &str) -> String {
        format!("v1:{}:{}", source_id, vin.to_uppercase())
    }
}
