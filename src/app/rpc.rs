use anyhow::Result;
use tracing::error;

use crate::domain::models::Transaction;
use crate::app::orchestrator::Orchestrator;
use crate::schema_capnp::tax_engine;
use crate::domain::traits::{ProfileRepositoryTrait, ProfileCacheTrait};

pub struct TaxEngineImpl<R, C>
where
    R: ProfileRepositoryTrait + Clone + Send + Sync + 'static,
    C: ProfileCacheTrait + Clone + Send + Sync + 'static,
{
    pub orchestrator: Orchestrator<R, C>,
}

impl<R, C> TaxEngineImpl<R, C>
where
    R: ProfileRepositoryTrait + Clone + Send + Sync + 'static,
    C: ProfileCacheTrait + Clone + Send + Sync + 'static,
{
    pub fn new(orchestrator: Orchestrator<R, C>) -> Self {
        Self { orchestrator }
    }
}

impl<R, C> tax_engine::Server for TaxEngineImpl<R, C>
where
    R: ProfileRepositoryTrait + Clone + Send + Sync + 'static,
    C: ProfileCacheTrait + Clone + Send + Sync + 'static,
{
    fn calculate(
        self: capnp::capability::Rc<Self>,
        params: tax_engine::CalculateParams,
        mut results: tax_engine::CalculateResults,
    ) -> impl futures_util::Future<Output = Result<(), capnp::Error>> + 'static {
        let orchestrator = self.orchestrator.clone();
        
        capnp::capability::Promise::from_future(async move {
            let request = params.get()?;
            let tx_req = request.get_tx()?;

            let client_id = tx_req.get_client_id()?.to_string()?;
            let amount = tx_req.get_amount();
            let jurisdiction = tx_req.get_jurisdiction()?.to_string()?;
            let product = tx_req.get_product()?.to_string()?;

            let tx = Transaction {
                amount,
                product,
                jurisdiction,
                client_id,
                date: chrono::Local::now().date_naive(),
            };

            match orchestrator.process_calculation(tx).await {
                Ok(breakdowns) => {
                    let mut response = results.get().init_response();
                    let total: f64 = breakdowns.iter().map(|b| b.amount).sum();
                    response.set_total_amount(total);

                    let mut list = response.init_breakdown(breakdowns.len() as u32);
                    for (i, b) in breakdowns.iter().enumerate() {
                        let mut detail = list.reborrow().get(i as u32);
                        detail.set_tax_type(format!("{:?}", b.tax_type));
                        detail.set_base(b.base);
                        detail.set_rate(b.rate);
                        detail.set_amount(b.amount);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Calculation error: {:?}", e);
                    Err(capnp::Error::failed(format!("Calculation failed: {}", e)))
                }
            }
        })
    }
}
