use crate::domain::models::{Transaction, Profile, TaxBreakdown, TaxType};

#[derive(Debug, Clone)]
pub struct IVACalculator;

impl IVACalculator {
    pub fn calculate(&self, tx: &Transaction, profile: &Profile, jurisdiction_rate: f64) -> TaxBreakdown {
        let rate = if profile.fiscal_category == "RESPONSABLE_INSCRIPTO" {
            jurisdiction_rate
        } else {
            0.0 // Monotributo doesn't calculate IVA for this POC
        };

        let amount = tx.amount * rate;

        TaxBreakdown {
            tax_type: TaxType::IVA,
            base: tx.amount,
            rate,
            amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Transaction, Profile};
    use chrono::Local;

    #[test]
    fn test_calculate_iva_responsable_inscripto() {
        let calculator = IVACalculator;
        let tx = Transaction {
            amount: 100.0,
            product: "PROD".to_string(),
            jurisdiction: "BSAS".to_string(),
            client_id: "c1".to_string(),
            date: Local::now().date_naive(),
        };
        let profile = Profile {
            client_id: "c1".to_string(),
            fiscal_category: "RESPONSABLE_INSCRIPTO".to_string(),
            config: serde_json::json!({}),
        };

        let res = calculator.calculate(&tx, &profile, 0.21);
        assert_eq!(res.rate, 0.21);
        assert_eq!(res.amount, 21.0);
    }

    #[test]
    fn test_calculate_iva_monotributo() {
        let calculator = IVACalculator;
        let tx = Transaction {
            amount: 100.0,
            product: "PROD".to_string(),
            jurisdiction: "BSAS".to_string(),
            client_id: "c1".to_string(),
            date: Local::now().date_naive(),
        };
        let profile = Profile {
            client_id: "c1".to_string(),
            fiscal_category: "MONOTRIBUTO".to_string(),
            config: serde_json::json!({}),
        };

        let res = calculator.calculate(&tx, &profile, 0.21);
        assert_eq!(res.rate, 0.0);
        assert_eq!(res.amount, 0.0);
    }

    #[test]
    fn test_calculate_iva_tdf() {
        let calculator = IVACalculator;
        let tx = Transaction {
            amount: 100.0,
            product: "PROD".to_string(),
            jurisdiction: "TDF".to_string(),
            client_id: "c1".to_string(),
            date: Local::now().date_naive(),
        };
        let profile = Profile {
            client_id: "c1".to_string(),
            fiscal_category: "RESPONSABLE_INSCRIPTO".to_string(),
            config: serde_json::json!({}),
        };

        let res = calculator.calculate(&tx, &profile, 0.105);
        assert_eq!(res.rate, 0.105);
        assert_eq!(res.amount, 10.5);
    }
}
