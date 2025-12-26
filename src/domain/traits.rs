use anyhow::Result;
use async_trait::async_trait;
use crate::domain::models::{Profile, IvaRate};

#[async_trait]
pub trait ProfileRepositoryTrait: Send + Sync {
    async fn get_by_id(&self, client_id: &str) -> Result<Option<Profile>>;
    async fn get_iva_rate(&self, jurisdiction: &str) -> Result<Option<IvaRate>>;
}

#[async_trait]
pub trait ProfileCacheTrait: Send + Sync {
    async fn get_by_id(&self, client_id: &str) -> Result<Option<Profile>>;
    async fn set(&self, profile: &Profile) -> Result<()>;
    async fn get_iva_rate(&self, jurisdiction: &str) -> Result<Option<IvaRate>>;
    async fn set_iva_rate(&self, rate: &IvaRate) -> Result<()>;
}
