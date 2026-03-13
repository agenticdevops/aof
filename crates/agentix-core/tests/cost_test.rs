use agentix_core::{
    CostRecord, CostSummary, ModelPricing, RunCostSummary,
    calculate_cost, default_model_pricing,
};
use chrono::Utc;

#[test]
fn test_cost_record_construction() {
    let now = Utc::now();
    let record = CostRecord {
        id: "rec-001".to_string(),
        agent_name: "my-agent".to_string(),
        run_id: "run-abc".to_string(),
        model: "claude-sonnet-4-6".to_string(),
        provider: "anthropic".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
        cost_usd: 0.0105,
        actual_cost_usd: None,
        recorded_at: now,
    };

    assert_eq!(record.agent_name, "my-agent");
    assert_eq!(record.input_tokens, 1000);
    assert_eq!(record.output_tokens, 500);
    assert_eq!(record.actual_cost_usd, None);

    // Round-trip serde
    let json = serde_json::to_string(&record).unwrap();
    let decoded: CostRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, "rec-001");
    assert_eq!(decoded.run_id, "run-abc");
    assert_eq!(decoded.cost_usd, 0.0105);
}

#[test]
fn test_cost_record_with_actual_cost() {
    let record = CostRecord {
        id: "rec-002".to_string(),
        agent_name: "my-agent".to_string(),
        run_id: "run-xyz".to_string(),
        model: "claude-3-5-sonnet-20241022".to_string(),
        provider: "anthropic".to_string(),
        input_tokens: 2000,
        output_tokens: 800,
        cost_usd: 0.018,
        actual_cost_usd: Some(0.014), // provider gave us actual cost
        recorded_at: Utc::now(),
    };

    assert_eq!(record.actual_cost_usd, Some(0.014));
    let json = serde_json::to_string(&record).unwrap();
    let decoded: CostRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.actual_cost_usd, Some(0.014));
}

#[test]
fn test_cost_summary_construction() {
    let summary = CostSummary {
        agent_name: "ops-agent".to_string(),
        total_runs: 10,
        total_input_tokens: 50000,
        total_output_tokens: 20000,
        total_cost_usd: 0.45,
        last_run_at: Some(Utc::now()),
    };

    assert_eq!(summary.agent_name, "ops-agent");
    assert_eq!(summary.total_runs, 10);
    assert_eq!(summary.total_input_tokens, 50000);
    assert_eq!(summary.total_output_tokens, 20000);
    assert!((summary.total_cost_usd - 0.45).abs() < 1e-9);
}

#[test]
fn test_run_cost_summary_construction() {
    let rcs = RunCostSummary {
        run_id: "run-001".to_string(),
        agent_name: "my-agent".to_string(),
        model: "gpt-4o".to_string(),
        input_tokens: 500,
        output_tokens: 200,
        cost_usd: 0.00325,
        started_at: Utc::now(),
    };

    assert_eq!(rcs.run_id, "run-001");
    assert_eq!(rcs.model, "gpt-4o");
    assert_eq!(rcs.input_tokens, 500);
}

#[test]
fn test_model_pricing_construction() {
    let pricing_anthropic = ModelPricing {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-6".to_string(),
        input_cost_per_1k: 0.003,
        output_cost_per_1k: 0.015,
    };
    assert_eq!(pricing_anthropic.input_cost_per_1k, 0.003);

    let pricing_openai = ModelPricing {
        provider: "openai".to_string(),
        model: "gpt-4o".to_string(),
        input_cost_per_1k: 0.0025,
        output_cost_per_1k: 0.01,
    };
    assert_eq!(pricing_openai.output_cost_per_1k, 0.01);

    let pricing_google = ModelPricing {
        provider: "google".to_string(),
        model: "gemini-2.0-flash".to_string(),
        input_cost_per_1k: 0.000075,
        output_cost_per_1k: 0.0003,
    };
    assert_eq!(pricing_google.input_cost_per_1k, 0.000075);
}

#[test]
fn test_calculate_cost_claude_sonnet() {
    // 1000 input + 500 output with claude-sonnet-4-6:
    // input: (1000 / 1000) * 0.003 = 0.003
    // output: (500 / 1000) * 0.015 = 0.0075
    // total: 0.0105
    let pricing = ModelPricing {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-6".to_string(),
        input_cost_per_1k: 0.003,
        output_cost_per_1k: 0.015,
    };
    let cost = calculate_cost(1000, 500, &pricing);
    let expected = 0.0105_f64;
    assert!(
        (cost - expected).abs() < f64::EPSILON * 100.0,
        "Expected {expected:.6}, got {cost:.6}"
    );
}

#[test]
fn test_calculate_cost_zero_tokens() {
    let pricing = ModelPricing {
        provider: "anthropic".to_string(),
        model: "claude-sonnet-4-6".to_string(),
        input_cost_per_1k: 0.003,
        output_cost_per_1k: 0.015,
    };
    let cost = calculate_cost(0, 0, &pricing);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_cost_gpt4o() {
    // 2000 input + 1000 output with gpt-4o:
    // input: (2000 / 1000) * 0.0025 = 0.005
    // output: (1000 / 1000) * 0.01 = 0.01
    // total: 0.015
    let pricing = ModelPricing {
        provider: "openai".to_string(),
        model: "gpt-4o".to_string(),
        input_cost_per_1k: 0.0025,
        output_cost_per_1k: 0.01,
    };
    let cost = calculate_cost(2000, 1000, &pricing);
    let expected = 0.015_f64;
    assert!(
        (cost - expected).abs() < f64::EPSILON * 100.0,
        "Expected {expected:.6}, got {cost:.6}"
    );
}

#[test]
fn test_default_model_pricing_table() {
    let table = default_model_pricing();
    // Must have at least 9 entries
    assert!(table.len() >= 9, "Expected at least 9 pricing entries, got {}", table.len());

    // Known entries must exist
    assert!(table.contains_key("anthropic/claude-sonnet-4-6"), "Missing anthropic/claude-sonnet-4-6");
    assert!(table.contains_key("openai/gpt-4o"), "Missing openai/gpt-4o");
    assert!(table.contains_key("google/gemini-2.0-flash"), "Missing google/gemini-2.0-flash");

    // Verify claude-sonnet-4-6 pricing
    let sonnet = &table["anthropic/claude-sonnet-4-6"];
    assert_eq!(sonnet.input_cost_per_1k, 0.003);
    assert_eq!(sonnet.output_cost_per_1k, 0.015);
}
