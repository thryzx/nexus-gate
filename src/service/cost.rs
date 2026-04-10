use crate::state::AppState;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pricing data for a model (per million tokens).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model: String,
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_read_per_mtok: f64,
    pub cache_creation_per_mtok: f64,
}

/// Usage data extracted from upstream response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageData {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cache_creation_input_tokens: u64,
}

/// Cost breakdown returned by calculate_cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_read_cost: f64,
    pub cache_creation_cost: f64,
    pub total_cost: f64,
}

/// Calculate USD cost from token usage (CRS-compatible).
pub fn calculate_cost(pricing: &ModelPricing, usage: &UsageData) -> CostBreakdown {
    let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * pricing.input_per_mtok;
    let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * pricing.output_per_mtok;
    let cache_read_cost =
        (usage.cache_read_input_tokens as f64 / 1_000_000.0) * pricing.cache_read_per_mtok;
    let cache_creation_cost =
        (usage.cache_creation_input_tokens as f64 / 1_000_000.0) * pricing.cache_creation_per_mtok;

    CostBreakdown {
        input_cost,
        output_cost,
        cache_read_cost,
        cache_creation_cost,
        total_cost: input_cost + output_cost + cache_read_cost + cache_creation_cost,
    }
}

/// Extract usage data from an upstream JSON response.
pub fn extract_usage(resp: &serde_json::Value) -> UsageData {
    let usage = &resp["usage"];
    UsageData {
        input_tokens: usage["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: usage["output_tokens"].as_u64().unwrap_or(0),
        cache_read_input_tokens: usage["cache_read_input_tokens"].as_u64().unwrap_or(0),
        cache_creation_input_tokens: usage["cache_creation_input_tokens"].as_u64().unwrap_or(0),
    }
}

/// Record a usage entry after a request completes.
pub async fn record_usage(
    state: &AppState,
    api_key_id: Uuid,
    account_id: Uuid,
    model: &str,
    usage: &UsageData,
    cost_usd: f64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO usage_logs (id, api_key_id, account_id, model, input_tokens, output_tokens, cost_usd)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(api_key_id)
    .bind(account_id)
    .bind(model)
    .bind(usage.input_tokens as i64)
    .bind(usage.output_tokens as i64)
    .bind(cost_usd)
    .execute(&state.db)
    .await?;

    Ok(())
}

/// Get the default pricing for well-known models (CRS parity).
pub fn default_pricing(model: &str) -> ModelPricing {
    // Strip [1m] suffix for lookup, matching CRS.
    let normalized = model.replace("[1m]", "").trim().to_string();
    let m = normalized.as_str();

    match m {
        _ if m.contains("opus") => ModelPricing {
            model: model.into(),
            input_per_mtok: 15.0,
            output_per_mtok: 75.0,
            cache_read_per_mtok: 1.5,
            cache_creation_per_mtok: 18.75,
        },
        _ if m.contains("sonnet") => ModelPricing {
            model: model.into(),
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
            cache_read_per_mtok: 0.3,
            cache_creation_per_mtok: 3.75,
        },
        _ if m.contains("haiku") => ModelPricing {
            model: model.into(),
            input_per_mtok: 0.8,
            output_per_mtok: 4.0,
            cache_read_per_mtok: 0.08,
            cache_creation_per_mtok: 1.0,
        },
        _ if m.contains("gpt-4o") => ModelPricing {
            model: model.into(),
            input_per_mtok: 2.5,
            output_per_mtok: 10.0,
            cache_read_per_mtok: 1.25,
            cache_creation_per_mtok: 0.0,
        },
        _ if m.contains("gemini") && m.contains("pro") => ModelPricing {
            model: model.into(),
            input_per_mtok: 1.25,
            output_per_mtok: 10.0,
            cache_read_per_mtok: 0.0,
            cache_creation_per_mtok: 0.0,
        },
        _ => ModelPricing {
            model: model.into(),
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
            cache_read_per_mtok: 0.0,
            cache_creation_per_mtok: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_calculation_basic() {
        let pricing = ModelPricing {
            model: "test".into(),
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
            cache_read_per_mtok: 0.0,
            cache_creation_per_mtok: 0.0,
        };
        let usage = UsageData {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            ..Default::default()
        };
        let cost = calculate_cost(&pricing, &usage);
        assert!((cost.total_cost - 18.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cost_zero_tokens() {
        let pricing = default_pricing("claude-opus");
        let usage = UsageData::default();
        let cost = calculate_cost(&pricing, &usage);
        assert!((cost.total_cost - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cost_with_cache_tokens() {
        let pricing = default_pricing("claude-opus-4");
        let usage = UsageData {
            input_tokens: 1_000_000,
            output_tokens: 100_000,
            cache_read_input_tokens: 500_000,
            cache_creation_input_tokens: 200_000,
        };
        let cost = calculate_cost(&pricing, &usage);

        // input: 1M * 15/M = 15.0
        // output: 0.1M * 75/M = 7.5
        // cache_read: 0.5M * 1.5/M = 0.75
        // cache_creation: 0.2M * 18.75/M = 3.75
        let expected = 15.0 + 7.5 + 0.75 + 3.75;
        assert!((cost.total_cost - expected).abs() < 0.001);
    }

    #[test]
    fn default_pricing_opus() {
        let p = default_pricing("claude-opus-4-6");
        assert_eq!(p.input_per_mtok, 15.0);
        assert_eq!(p.cache_creation_per_mtok, 18.75);
    }

    #[test]
    fn default_pricing_sonnet() {
        let p = default_pricing("claude-sonnet-4-20250514");
        assert_eq!(p.input_per_mtok, 3.0);
    }

    #[test]
    fn default_pricing_unknown_model() {
        let p = default_pricing("unknown-model");
        assert_eq!(p.input_per_mtok, 3.0);
    }

    #[test]
    fn extract_usage_from_response() {
        let resp = serde_json::json!({
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50,
                "cache_read_input_tokens": 30,
                "cache_creation_input_tokens": 20
            }
        });
        let usage = extract_usage(&resp);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_read_input_tokens, 30);
        assert_eq!(usage.cache_creation_input_tokens, 20);
    }

    #[test]
    fn extract_usage_missing_fields() {
        let resp = serde_json::json!({ "usage": {} });
        let usage = extract_usage(&resp);
        assert_eq!(usage.input_tokens, 0);
    }

    #[test]
    fn pricing_strips_1m_suffix() {
        let p = default_pricing("claude-opus-4[1m]");
        assert_eq!(p.input_per_mtok, 15.0);
    }
}
