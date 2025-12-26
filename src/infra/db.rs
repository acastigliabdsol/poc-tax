use crate::domain::models::Profile;
use crate::domain::traits::ProfileRepositoryTrait;
use anyhow::{Context, Result};
use sqlx::PgPool;
use async_trait::async_trait;

#[derive(Clone)]
pub struct ProfileRepository {
    pool: PgPool,
}

impl ProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileRepositoryTrait for ProfileRepository {
    async fn get_by_id(&self, client_id: &str) -> Result<Option<Profile>> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT client_id, fiscal_category, config FROM profiles WHERE client_id = $1"
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch profile from DB")?;

        Ok(row.map(|r| Profile {
            client_id: r.get("client_id"),
            fiscal_category: r.get("fiscal_category"),
            config: r.get("config"),
        }))
    }

    async fn get_iva_rate(&self, jurisdiction: &str) -> Result<Option<crate::domain::models::IvaRate>> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT jurisdiction, rate FROM iva_rates WHERE jurisdiction = $1"
        )
        .bind(jurisdiction)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch IVA rate from DB")?;

        Ok(row.map(|r| crate::domain::models::IvaRate {
            jurisdiction: r.get("jurisdiction"),
            rate: r.get("rate"),
        }))
    }
}
