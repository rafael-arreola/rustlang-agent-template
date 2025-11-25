use anyhow::Result;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    User,
    System,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone)]
pub struct RedisProvider {
    client: redis::Client,
    base_path: String,
    ttl: usize,
}

impl RedisProvider {
    pub fn new() -> Result<Self> {
        let config = crate::envs::get();
        let redis_url = &config.redis_url;
        let base_path = config.redis_base_path.clone();
        let ttl = config.session_ttl as usize;

        let client = redis::Client::open(redis_url.as_str())?;

        Ok(Self {
            client,
            base_path,
            ttl,
        })
    }

    fn get_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.base_path, session_id)
    }

    pub async fn get_history(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = self.get_key(session_id);
        let history_json: Option<String> = con.get(&key).await?;

        match history_json {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Ok(Vec::new()),
        }
    }

    pub async fn add_message(&self, session_id: &str, role: Role, content: String) -> Result<()> {
        // We re-fetch to ensure we append to the latest state (optimistic locking would be better but simple get-set is fine for now)
        let mut history = self.get_history(session_id).await?;
        history.push(ChatMessage { role, content });

        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = self.get_key(session_id);
        let json = serde_json::to_string(&history)?;

        con.set_ex::<_, _, ()>(&key, json, self.ttl as u64).await?;

        Ok(())
    }
    
    // Helper to add multiple messages at once to avoid race conditions in the naive implementation
    pub async fn add_messages(&self, session_id: &str, messages: Vec<ChatMessage>) -> Result<()> {
        let mut history = self.get_history(session_id).await?;
        history.extend(messages);

        let mut con = self.client.get_multiplexed_async_connection().await?;
        let key = self.get_key(session_id);
        let json = serde_json::to_string(&history)?;

        con.set_ex::<_, _, ()>(&key, json, self.ttl as u64).await?;

        Ok(())
    }
}
