use crate::domain::models::Profile;
use crate::domain::traits::{ProfileRepositoryTrait, ProfileCacheTrait};
use anyhow::Result;
use tracing::{debug, info};

#[derive(Clone)]
pub struct ProfileResolver<R, C> 
where 
    R: ProfileRepositoryTrait,
    C: ProfileCacheTrait,
{
    db: R,
    cache: C,
}

impl<R, C> ProfileResolver<R, C>
where 
    R: ProfileRepositoryTrait,
    C: ProfileCacheTrait,
{
    pub fn new(db: R, cache: C) -> Self {
        Self { db, cache }
    }

    pub async fn resolve(&self, client_id: &str) -> Result<Option<Profile>> {
        // Try L2 Cache (Redis)
        if let Some(profile) = self.cache.get_by_id(client_id).await? {
            debug!("Cache hit for client_id: {}", client_id);
            return Ok(Some(profile));
        }

        info!("Cache miss for client_id: {}. Fetching from DB...", client_id);
        
        // Try DB
        if let Some(profile) = self.db.get_by_id(client_id).await? {
            // Populate Cache
            self.cache.set(&profile).await?;
            debug!("Populated cache for client_id: {}", client_id);
            return Ok(Some(profile));
        }

        Ok(None)
    }

    pub async fn resolve_iva_rate(&self, jurisdiction: &str) -> Result<f64> {
        // Try Cache
        if let Some(rate_info) = self.cache.get_iva_rate(jurisdiction).await? {
            debug!("Cache hit for IVA rate in jurisdiction: {}", jurisdiction);
            return Ok(rate_info.rate);
        }

        info!("Cache miss for IVA rate in jurisdiction: {}. Fetching from DB...", jurisdiction);

        // Try DB for specific jurisdiction
        if let Some(rate_info) = self.db.get_iva_rate(jurisdiction).await? {
            self.cache.set_iva_rate(&rate_info).await?;
            return Ok(rate_info.rate);
        }

        // Fallback to DEFAULT
        if let Some(rate_info) = self.db.get_iva_rate("DEFAULT").await? {
             // Cache the specific jurisdiction with the default rate to avoid constant misses
            let specific_rate = crate::domain::models::IvaRate {
                jurisdiction: jurisdiction.to_string(),
                rate: rate_info.rate,
            };
            self.cache.set_iva_rate(&specific_rate).await?;
            return Ok(rate_info.rate);
        }

        Ok(0.21) // Hardcoded safety fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, IvaRate};
    use mockall::mock;
    use async_trait::async_trait;

    mock! {
        pub Repo {}
        #[async_trait]
        impl ProfileRepositoryTrait for Repo {
            async fn get_by_id(&self, client_id: &str) -> Result<Option<Profile>>;
            async fn get_iva_rate(&self, jurisdiction: &str) -> Result<Option<IvaRate>>;
        }
    }

    mock! {
        pub Cache {}
        #[async_trait]
        impl ProfileCacheTrait for Cache {
            async fn get_by_id(&self, client_id: &str) -> Result<Option<Profile>>;
            async fn set(&self, profile: &Profile) -> Result<()>;
            async fn get_iva_rate(&self, jurisdiction: &str) -> Result<Option<IvaRate>>;
            async fn set_iva_rate(&self, rate: &IvaRate) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_resolve_profile_cache_hit() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        let profile = Profile {
            client_id: "c1".to_string(),
            fiscal_category: "RI".to_string(),
            config: serde_json::json!({}),
        };

        let p_clone = profile.clone();
        mock_cache.expect_get_by_id()
            .with(mockall::predicate::eq("c1"))
            .times(1)
            .returning(move |_| Ok(Some(p_clone.clone())));

        mock_db.expect_get_by_id().times(0);

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let res = resolver.resolve("c1").await.unwrap();

        assert!(res.is_some());
        assert_eq!(res.unwrap().client_id, "c1");
    }

    #[tokio::test]
    async fn test_resolve_profile_cache_miss_db_hit() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        let profile = Profile {
            client_id: "c1".to_string(),
            fiscal_category: "RI".to_string(),
            config: serde_json::json!({}),
        };

        mock_cache.expect_get_by_id().returning(|_| Ok(None));
        
        let p_clone = profile.clone();
        mock_db.expect_get_by_id().returning(move |_| Ok(Some(p_clone.clone())));
        
        mock_cache.expect_set().times(1).returning(|_| Ok(()));

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let res = resolver.resolve("c1").await.unwrap();

        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_resolve_iva_rate_cache_hit() {
        let mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        mock_cache.expect_get_iva_rate()
            .returning(|_| Ok(Some(IvaRate { jurisdiction: "J1".to_string(), rate: 0.1 })));

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let rate = resolver.resolve_iva_rate("J1").await.unwrap();

        assert_eq!(rate, 0.1);
    }

    #[tokio::test]
    async fn test_resolve_iva_rate_cache_miss_db_hit() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        mock_cache.expect_get_iva_rate().returning(|_| Ok(None));
        mock_db.expect_get_iva_rate()
            .with(mockall::predicate::eq("J1"))
            .returning(|_| Ok(Some(IvaRate { jurisdiction: "J1".to_string(), rate: 0.15 })));
        mock_cache.expect_set_iva_rate().times(1).returning(|_| Ok(()));

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let rate = resolver.resolve_iva_rate("J1").await.unwrap();

        assert_eq!(rate, 0.15);
    }

    #[tokio::test]
    async fn test_resolve_iva_rate_fallback_to_default() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        mock_cache.expect_get_iva_rate().returning(|_| Ok(None));
        mock_db.expect_get_iva_rate()
            .with(mockall::predicate::eq("UNKNOWN"))
            .returning(|_| Ok(None));
        mock_db.expect_get_iva_rate()
            .with(mockall::predicate::eq("DEFAULT"))
            .returning(|_| Ok(Some(IvaRate { jurisdiction: "DEFAULT".to_string(), rate: 0.21 })));
        
        mock_cache.expect_set_iva_rate().times(1).returning(|_| Ok(()));

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let rate = resolver.resolve_iva_rate("UNKNOWN").await.unwrap();

        assert_eq!(rate, 0.21);
    }

    #[tokio::test]
    async fn test_resolve_iva_rate_total_fallback() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        mock_cache.expect_get_iva_rate().returning(|_| Ok(None));
        mock_db.expect_get_iva_rate().returning(|_| Ok(None));

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let rate = resolver.resolve_iva_rate("ANY").await.unwrap();

        assert_eq!(rate, 0.21);
    }

    #[tokio::test]
    async fn test_resolve_profile_not_found() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        mock_cache.expect_get_by_id().returning(|_| Ok(None));
        mock_db.expect_get_by_id().returning(|_| Ok(None));

        let resolver = ProfileResolver::new(mock_db, mock_cache);
        let res = resolver.resolve("non_existent").await.unwrap();

        assert!(res.is_none());
    }
}
