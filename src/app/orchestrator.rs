use crate::domain::models::{Transaction, TaxBreakdown};
use crate::app::resolver::ProfileResolver;
use crate::domain::calculators::IVACalculator;
use crate::domain::traits::{ProfileRepositoryTrait, ProfileCacheTrait};
use anyhow::{Context, Result};
use tracing::info;

#[derive(Clone)]
pub struct Orchestrator<R, C>
where 
    R: ProfileRepositoryTrait,
    C: ProfileCacheTrait,
{
    pub profile_resolver: ProfileResolver<R, C>,
    pub iva_calculator: IVACalculator,
}

impl<R, C> Orchestrator<R, C>
where 
    R: ProfileRepositoryTrait,
    C: ProfileCacheTrait,
{
    pub fn new(profile_resolver: ProfileResolver<R, C>, iva_calculator: IVACalculator) -> Self {
        Self {
            profile_resolver,
            iva_calculator,
        }
    }

    pub async fn process_calculation(&self, tx: Transaction) -> Result<Vec<TaxBreakdown>> {
        info!("Processing calculation for client: {}", tx.client_id);

        let profile = self.profile_resolver
            .resolve(&tx.client_id)
            .await?
            .context("Profile not found")?;

        // Resolve IVA Rate based on jurisdiction
        let jurisdiction_rate = self.profile_resolver
            .resolve_iva_rate(&tx.jurisdiction)
            .await?;

        let mut breakdowns = Vec::new();

        // Calculate IVA
        let iva = self.iva_calculator.calculate(&tx, &profile, jurisdiction_rate);
        breakdowns.push(iva);

        Ok(breakdowns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, IvaRate};
    use mockall::mock;
    use async_trait::async_trait;
    use chrono::Local;

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
    async fn test_process_calculation_success() {
        let mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        let profile = Profile {
            client_id: "c1".to_string(),
            fiscal_category: "RESPONSABLE_INSCRIPTO".to_string(),
            config: serde_json::json!({}),
        };

        mock_cache.expect_get_by_id().returning(move |_| Ok(Some(profile.clone())));
        mock_cache.expect_get_iva_rate().returning(|_| Ok(Some(IvaRate { jurisdiction: "J1".to_string(), rate: 0.21 })));

        let profile_resolver = ProfileResolver::new(mock_db, mock_cache);
        let orchestrator = Orchestrator::new(profile_resolver, IVACalculator);

        let tx = Transaction {
            amount: 100.0,
            product: "P".to_string(),
            jurisdiction: "J1".to_string(),
            client_id: "c1".to_string(),
            date: Local::now().date_naive(),
        };

        let res = orchestrator.process_calculation(tx).await.unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].amount, 21.0);
    }

    #[tokio::test]
    async fn test_process_calculation_profile_not_found() {
        let mut mock_db = MockRepo::new();
        let mut mock_cache = MockCache::new();

        mock_cache.expect_get_by_id().returning(|_| Ok(None));
        mock_db.expect_get_by_id().returning(|_| Ok(None));

        let profile_resolver = ProfileResolver::new(mock_db, mock_cache);
        let orchestrator = Orchestrator::new(profile_resolver, IVACalculator);

        let tx = Transaction {
            amount: 100.0,
            product: "P".to_string(),
            jurisdiction: "J1".to_string(),
            client_id: "unknown".to_string(),
            date: Local::now().date_naive(),
        };

        let res = orchestrator.process_calculation(tx).await;
        assert!(res.is_err());
    }
}
