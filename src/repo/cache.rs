use anyhow::Result;
use redis::AsyncCommands;

/// Generic Redis cache operations.
pub struct Cache {
    conn: redis::aio::MultiplexedConnection,
}

impl Cache {
    pub async fn new(client: &redis::Client) -> Result<Self> {
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self { conn })
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<String>> {
        let val: Option<String> = self.conn.get(key).await?;
        Ok(val)
    }

    pub async fn set(&mut self, key: &str, value: &str, ttl_secs: u64) -> Result<()> {
        self.conn.set_ex::<_, _, ()>(key, value, ttl_secs).await?;
        Ok(())
    }

    pub async fn del(&mut self, key: &str) -> Result<()> {
        self.conn.del::<_, ()>(key).await?;
        Ok(())
    }

    pub async fn incr(&mut self, key: &str, ttl_secs: u64) -> Result<i64> {
        let val: i64 = self.conn.incr(key, 1i64).await?;
        if val == 1 {
            self.conn.expire::<_, ()>(key, ttl_secs as i64).await?;
        }
        Ok(val)
    }
}
