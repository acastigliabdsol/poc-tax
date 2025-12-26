use crate::domain::models::Profile;
use crate::domain::traits::ProfileCacheTrait;
use anyhow::{Context, Result};
use redis::AsyncCommands;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Clone)]
pub struct ProfileCache {
    client: Arc<redis::Client>,
}

impl ProfileCache {
    pub fn new(client: Arc<redis::Client>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ProfileCacheTrait for ProfileCache {
    async fn get_by_id(&self, client_id: &str) -> Result<Option<Profile>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("profile:{}", client_id);
        
        let cached: Option<String> = conn.get(&key).await.context("Failed to get from Redis")?;
        
        match cached {
            Some(json) => {
                let profile: Profile = serde_json::from_str(&json).context("Failed to parse cached profile")?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, profile: &Profile) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("profile:{}", profile.client_id);
        let json = serde_json::to_string(profile).context("Failed to serialize profile for cache")?;
        
        // TTL 30 minutes as per RT-002
        let _: () = conn.set_ex(key, json, 1800).await.context("Failed to set in Redis")?;
        Ok(())
    }

    async fn get_iva_rate(&self, jurisdiction: &str) -> Result<Option<crate::domain::models::IvaRate>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("iva_rate:{}", jurisdiction);
        
        let cached: Option<String> = conn.get(&key).await.context("Failed to get IVA rate from Redis")?;
        
        match cached {
            Some(json) => {
                let rate: crate::domain::models::IvaRate = serde_json::from_str(&json).context("Failed to parse cached IVA rate")?;
                Ok(Some(rate))
            }
            None => Ok(None),
        }
    }

    async fn set_iva_rate(&self, rate: &crate::domain::models::IvaRate) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("iva_rate:{}", rate.jurisdiction);
        let json = serde_json::to_string(rate).context("Failed to serialize IVA rate for cache")?;
        
        let _: () = conn.set_ex(key, json, 1800).await.context("Failed to set IVA rate in Redis")?;
        Ok(())
    }
}
