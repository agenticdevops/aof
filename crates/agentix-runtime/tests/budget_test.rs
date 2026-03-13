use agentix_core::{BudgetStopReason, ModelComplexityScore, ModelTier};

#[test]
fn test_complexity_score_simple() {
    let score = ModelComplexityScore::score("hello");
    assert!(score <= 30, "Simple message 'hello' should score ≤30 (Flash), got {score}");
    assert_eq!(ModelComplexityScore::tier(score), ModelTier::Flash);
}

#[test]
fn test_complexity_score_complex() {
    let msg = "Please analyze and research the architecture to investigate performance optimizations and explain why the current design causes issues";
    let score = ModelComplexityScore::score(msg);
    assert!(
        score > 30,
        "Complex message should score >30 (Standard or Pro), got {score}"
    );
}

#[test]
fn test_model_tier_routing() {
    // Short message → Flash
    let flash_score = ModelComplexityScore::score("hi");
    assert_eq!(ModelComplexityScore::tier(flash_score), ModelTier::Flash);

    // Score 40 (boundary check): use a long message with many keywords
    // A message with length_score=40 alone would need 2000+ chars
    // Direct scoring test: 8 keyword hits × 5 = 40 points → Standard tier
    let eight_kw_msg = "analyze research compare explain implement design optimize debug this system";
    let kw_score = ModelComplexityScore::score(eight_kw_msg);
    assert!(kw_score > 30, "8-keyword message should exceed Flash threshold (30), got {kw_score}");

    // Direct tier mapping
    assert_eq!(ModelComplexityScore::tier(0), ModelTier::Flash);
    assert_eq!(ModelComplexityScore::tier(30), ModelTier::Flash);
    assert_eq!(ModelComplexityScore::tier(31), ModelTier::Standard);
    assert_eq!(ModelComplexityScore::tier(70), ModelTier::Standard);
    assert_eq!(ModelComplexityScore::tier(71), ModelTier::Pro);
    assert_eq!(ModelComplexityScore::tier(100), ModelTier::Pro);
}

#[test]
fn test_budget_stop_reason_serde() {
    let reason = BudgetStopReason::DailyLimitExceeded {
        limit_usd: 0.5,
        spent_usd: 0.52,
    };
    let json = serde_json::to_string(&reason).unwrap();
    // Should serialize with "reason": "daily_limit_exceeded"
    assert!(
        json.contains("\"daily_limit_exceeded\""),
        "Expected daily_limit_exceeded tag, got: {json}"
    );
    assert!(json.contains("0.5"));
    assert!(json.contains("0.52"));

    // Round-trip
    let decoded: BudgetStopReason = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, reason);

    // Token limit variant
    let token_reason = BudgetStopReason::TokenLimitExceeded {
        limit: 4000,
        used: 4250,
    };
    let token_json = serde_json::to_string(&token_reason).unwrap();
    assert!(
        token_json.contains("\"token_limit_exceeded\""),
        "Expected token_limit_exceeded tag, got: {token_json}"
    );

    let decoded_token: BudgetStopReason = serde_json::from_str(&token_json).unwrap();
    assert_eq!(decoded_token, token_reason);
}
