use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One LLM call cost record — stored per API call.
///
/// Tracks both estimated cost (from pricing table) and actual cost
/// from the provider API response when available (COST-07).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// Unique record ID (UUID v4)
    pub id: String,
    /// Agent that made the call
    pub agent_name: String,
    /// Run this call belongs to
    pub run_id: String,
    /// Model used (e.g., "claude-sonnet-4-6")
    pub model: String,
    /// Provider (e.g., "anthropic")
    pub provider: String,
    /// Input tokens consumed
    pub input_tokens: u64,
    /// Output tokens produced
    pub output_tokens: u64,
    /// Estimated cost in USD (from pricing table)
    pub cost_usd: f64,
    /// Actual cost from provider API response (COST-07 — preferred over estimate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_cost_usd: Option<f64>,
    /// When this record was created
    pub recorded_at: DateTime<Utc>,
}

/// Per-agent cost summary — aggregated across all runs.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostSummary {
    /// Agent name
    pub agent_name: String,
    /// Total number of distinct runs
    pub total_runs: u64,
    /// Total input tokens across all runs
    pub total_input_tokens: u64,
    /// Total output tokens across all runs
    pub total_output_tokens: u64,
    /// Total cost in USD (uses actual when available, estimated otherwise)
    pub total_cost_usd: f64,
    /// Most recent run timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<DateTime<Utc>>,
}

/// Per-run cost summary — aggregated across all LLM calls in one run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCostSummary {
    /// Run ID
    pub run_id: String,
    /// Agent name
    pub agent_name: String,
    /// Primary model used in this run
    pub model: String,
    /// Total input tokens in this run
    pub input_tokens: u64,
    /// Total output tokens in this run
    pub output_tokens: u64,
    /// Total cost for this run in USD
    pub cost_usd: f64,
    /// When this run started
    pub started_at: DateTime<Utc>,
}

/// Model pricing entry — cost per 1,000 tokens in USD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Provider name (e.g., "anthropic")
    pub provider: String,
    /// Model name (e.g., "claude-sonnet-4-6")
    pub model: String,
    /// USD per 1,000 input tokens
    pub input_cost_per_1k: f64,
    /// USD per 1,000 output tokens
    pub output_cost_per_1k: f64,
}

/// Calculate estimated cost in USD for a given token count and pricing.
///
/// # Arguments
/// * `input_tokens` — Number of input tokens
/// * `output_tokens` — Number of output tokens
/// * `pricing` — Pricing entry for the model
///
/// # Returns
/// Estimated cost in USD
pub fn calculate_cost(input_tokens: u64, output_tokens: u64, pricing: &ModelPricing) -> f64 {
    let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
    let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
    input_cost + output_cost
}

/// Default model pricing table keyed by "provider/model".
///
/// Prices are in USD per 1,000 tokens. Updated as of 2026-03.
/// Sources: Anthropic, OpenAI, Google pricing pages.
pub fn default_model_pricing() -> HashMap<String, ModelPricing> {
    let entries = vec![
        // Anthropic
        ModelPricing {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
        ModelPricing {
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        },
        ModelPricing {
            provider: "anthropic".to_string(),
            model: "claude-3-5-haiku-20241022".to_string(),
            input_cost_per_1k: 0.0008,
            output_cost_per_1k: 0.004,
        },
        ModelPricing {
            provider: "anthropic".to_string(),
            model: "claude-3-opus-20240229".to_string(),
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
        },
        // OpenAI
        ModelPricing {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            input_cost_per_1k: 0.0025,
            output_cost_per_1k: 0.01,
        },
        ModelPricing {
            provider: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0006,
        },
        // Google
        ModelPricing {
            provider: "google".to_string(),
            model: "gemini-2.0-flash".to_string(),
            input_cost_per_1k: 0.000075,
            output_cost_per_1k: 0.0003,
        },
        ModelPricing {
            provider: "google".to_string(),
            model: "gemini-2.5-pro".to_string(),
            input_cost_per_1k: 0.00125,
            output_cost_per_1k: 0.01,
        },
        // Groq
        ModelPricing {
            provider: "groq".to_string(),
            model: "llama-3.3-70b-versatile".to_string(),
            input_cost_per_1k: 0.00059,
            output_cost_per_1k: 0.00079,
        },
    ];

    entries
        .into_iter()
        .map(|p| (format!("{}/{}", p.provider, p.model), p))
        .collect()
}

/// Budget configuration for an agent — controls spending limits (Phase 17).
///
/// Set in agent YAML under `spec.budget:` (GitAgent) or top-level `budget:` (flat YAML).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetConfig {
    /// Stop agent if daily USD spend exceeds this limit (COST-04).
    /// The run is blocked from starting when the daily limit is reached.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_limit_usd: Option<f64>,
    /// Stop run if total tokens (input + output combined) in this run exceed this limit (COST-05).
    /// The ReAct loop stops after the current iteration when exceeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_run: Option<u64>,
}

/// Reason an agent run was stopped by a budget limit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum BudgetStopReason {
    /// Daily USD spend limit exceeded before run started (COST-04)
    DailyLimitExceeded { limit_usd: f64, spent_usd: f64 },
    /// Per-run token limit exceeded mid-execution (COST-05)
    TokenLimitExceeded { limit: u64, used: u64 },
}

/// Model complexity tier for smart model routing (CORE-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// Score 0-30: simple Q&A, short messages → cheapest models
    Flash,
    /// Score 31-70: moderate complexity tasks
    Standard,
    /// Score 71-100: complex research, long context, analysis
    Pro,
}

/// Scores a message for complexity (0-100) to enable smart model routing.
///
/// Higher score = more complex = routes to more capable (and expensive) model tier.
///
/// # Algorithm
/// 1. Length score: `min(len / 50, 40)` → up to 40 points for long inputs
/// 2. Keyword score: count complex keywords × 5, capped at 40 points
/// 3. Question depth: "why"/"how" occurrences × 5, capped at 20 points
pub struct ModelComplexityScore;

impl ModelComplexityScore {
    /// Score a message for complexity (0-100).
    pub fn score(input: &str) -> u8 {
        let lower = input.to_lowercase();

        // Length component (up to 40 points)
        let length_score = (input.len() / 50).min(40) as u8;

        // Keyword component (up to 40 points)
        let keywords = [
            "analyze",
            "research",
            "compare",
            "explain",
            "implement",
            "design",
            "architecture",
            "optimize",
            "debug",
            "investigate",
            "evaluate",
        ];
        let keyword_count: u8 = keywords
            .iter()
            .map(|kw| lower.matches(kw).count() as u8)
            .sum::<u8>()
            .min(8); // cap at 8 keyword hits
        let keyword_score = (keyword_count * 5).min(40);

        // Question depth (up to 20 points)
        let why_count = lower.matches("why").count().min(2) as u8;
        let how_count = lower.matches("how").count().min(2) as u8;
        let question_score = ((why_count + how_count) * 5).min(20);

        (length_score + keyword_score + question_score).min(100)
    }

    /// Map a complexity score to a model tier.
    pub fn tier(score: u8) -> ModelTier {
        match score {
            0..=30 => ModelTier::Flash,
            31..=70 => ModelTier::Standard,
            _ => ModelTier::Pro,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_cost_basic() {
        let pricing = ModelPricing {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
        };
        let cost = calculate_cost(1000, 500, &pricing);
        assert!((cost - 0.0105).abs() < f64::EPSILON * 100.0);
    }

    #[test]
    fn test_complexity_score_simple() {
        assert!(ModelComplexityScore::score("hi") <= 30);
        assert_eq!(ModelComplexityScore::tier(ModelComplexityScore::score("hi")), ModelTier::Flash);
    }

    #[test]
    fn test_complexity_score_complex() {
        // 7 keywords × 5 = 35 points → Standard tier (> 30)
        let msg = "analyze research compare explain implement design optimize this system architecture";
        let score = ModelComplexityScore::score(msg);
        assert!(score > 30, "Expected score > 30, got {score}");
        assert_eq!(ModelComplexityScore::tier(score), ModelTier::Standard);
    }
}
