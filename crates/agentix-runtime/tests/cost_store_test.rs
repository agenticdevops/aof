use agentix_core::{CostRecord};
use agentix_runtime::CostStore;
use chrono::Utc;

fn make_record(
    id: &str,
    agent: &str,
    run_id: &str,
    model: &str,
    provider: &str,
    input_tokens: u64,
    output_tokens: u64,
    cost_usd: f64,
    actual_cost_usd: Option<f64>,
) -> CostRecord {
    CostRecord {
        id: id.to_string(),
        agent_name: agent.to_string(),
        run_id: run_id.to_string(),
        model: model.to_string(),
        provider: provider.to_string(),
        input_tokens,
        output_tokens,
        cost_usd,
        actual_cost_usd,
        recorded_at: Utc::now(),
    }
}

#[test]
fn test_insert_and_get_run_summary() {
    let store = CostStore::open(":memory:").unwrap();

    // Two calls in the same run
    store.insert_record(&make_record(
        "r1-call1", "agent-a", "run-001", "claude-sonnet-4-6", "anthropic",
        1000, 400, 0.009, None,
    )).unwrap();

    store.insert_record(&make_record(
        "r1-call2", "agent-a", "run-001", "claude-sonnet-4-6", "anthropic",
        500, 200, 0.0045, None,
    )).unwrap();

    let summary = store.get_run_summary("run-001").unwrap();
    assert_eq!(summary.run_id, "run-001");
    assert_eq!(summary.agent_name, "agent-a");
    assert_eq!(summary.input_tokens, 1500);
    assert_eq!(summary.output_tokens, 600);
    // total cost = 0.009 + 0.0045 = 0.0135
    assert!((summary.cost_usd - 0.0135).abs() < 1e-9, "Expected 0.0135, got {}", summary.cost_usd);
}

#[test]
fn test_get_agent_summary() {
    let store = CostStore::open(":memory:").unwrap();

    // Run 1
    store.insert_record(&make_record(
        "a1", "my-agent", "run-A", "gpt-4o", "openai",
        2000, 800, 0.013, None,
    )).unwrap();

    // Run 2
    store.insert_record(&make_record(
        "a2", "my-agent", "run-B", "gpt-4o", "openai",
        1500, 600, 0.00975, None,
    )).unwrap();

    let summary = store.get_agent_summary("my-agent").unwrap();
    assert_eq!(summary.agent_name, "my-agent");
    assert_eq!(summary.total_runs, 2, "Expected 2 distinct runs");
    assert_eq!(summary.total_input_tokens, 3500);
    assert_eq!(summary.total_output_tokens, 1400);
    let expected_cost = 0.013 + 0.00975;
    assert!(
        (summary.total_cost_usd - expected_cost).abs() < 1e-9,
        "Expected {expected_cost:.6}, got {:.6}", summary.total_cost_usd
    );
}

#[test]
fn test_list_all_summaries() {
    let store = CostStore::open(":memory:").unwrap();

    store.insert_record(&make_record("s1", "agent-x", "run-x1", "gemini-2.0-flash", "google", 500, 200, 0.001, None)).unwrap();
    store.insert_record(&make_record("s2", "agent-y", "run-y1", "gpt-4o-mini", "openai", 300, 100, 0.00006, None)).unwrap();
    store.insert_record(&make_record("s3", "agent-z", "run-z1", "claude-3-5-haiku-20241022", "anthropic", 1000, 400, 0.0024, None)).unwrap();

    let summaries = store.list_all_summaries().unwrap();
    assert_eq!(summaries.len(), 3, "Expected 3 agent summaries");

    // Each summary should have a distinct agent name
    let names: Vec<&str> = summaries.iter().map(|s| s.agent_name.as_str()).collect();
    assert!(names.contains(&"agent-x"));
    assert!(names.contains(&"agent-y"));
    assert!(names.contains(&"agent-z"));
}

#[test]
fn test_actual_cost_precedence() {
    let store = CostStore::open(":memory:").unwrap();

    // cost_usd = 0.01 but actual_cost_usd = 0.008 (provider said so)
    store.insert_record(&make_record(
        "p1", "budget-agent", "run-p1", "claude-sonnet-4-6", "anthropic",
        1000, 500, 0.01, Some(0.008),
    )).unwrap();

    let summary = store.get_agent_summary("budget-agent").unwrap();
    // COALESCE should use 0.008 (actual) not 0.01 (estimated)
    assert!(
        (summary.total_cost_usd - 0.008).abs() < 1e-9,
        "Expected 0.008 (actual_cost_usd), got {}", summary.total_cost_usd
    );
}
