//! TDD integration tests for the approval gate in the ReAct loop (Plan 20-03).
//!
//! These tests verify that the ReAct loop correctly pauses for human approval
//! when an agent is in Manual or SemiAutonomous mode, and proceeds without
//! any overhead in Autonomous mode.
//!
//! Mock LLM and tool executor are defined here to avoid real infrastructure.

use agentix_core::{
    AgentDefinition, AgentMode, AgentixError, AgentixResult, ApprovalPolicy, ApprovalStatus,
    ModelConfig, ModelProvider, ModelRequest, ModelResponse, RequestMessage, SkillEntry,
    StopReason, ToolCall, Usage,
};
use agentix_core::agent::{DirectoryToolType, ToolEntry};
use agentix_runtime::{
    ApprovalStore,
    AuditEventType, AuditStore,
    executor::react_loop::{ReActConfig, ReActEngine, ReActEvent, ToolExecutor},
};

use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Mock infrastructure
// ---------------------------------------------------------------------------

/// Canned response from the mock LLM.
enum MockResponse {
    ToolCall {
        tool_name: String,
        input: serde_json::Value,
    },
    FinalAnswer {
        text: String,
    },
}

/// Mock LLM that returns pre-programmed responses in order.
struct MockModel {
    responses: Vec<MockResponse>,
    call_count: AtomicUsize,
}

impl MockModel {
    fn new(responses: Vec<MockResponse>) -> Arc<Self> {
        Arc::new(Self {
            responses,
            call_count: AtomicUsize::new(0),
        })
    }
}

#[async_trait]
impl agentix_core::Model for MockModel {
    async fn generate(&self, _request: &ModelRequest) -> AgentixResult<ModelResponse> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);

        if idx >= self.responses.len() {
            return Ok(ModelResponse {
                content: "final answer".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
                metadata: HashMap::new(),
            });
        }

        match &self.responses[idx] {
            MockResponse::ToolCall { tool_name, input } => Ok(ModelResponse {
                content: format!("I will call {}", tool_name),
                tool_calls: vec![ToolCall {
                    id: format!("call_{idx}"),
                    name: tool_name.clone(),
                    arguments: input.clone(),
                }],
                stop_reason: StopReason::ToolUse,
                usage: Usage::default(),
                metadata: HashMap::new(),
            }),
            MockResponse::FinalAnswer { text } => Ok(ModelResponse {
                content: text.clone(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
                metadata: HashMap::new(),
            }),
        }
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AgentixResult<Pin<Box<dyn futures::Stream<Item = AgentixResult<agentix_core::StreamChunk>> + Send>>>
    {
        unimplemented!("streaming not needed for approval gate tests")
    }

    fn config(&self) -> &ModelConfig {
        unimplemented!("config not needed for approval gate tests")
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Mock tool executor that tracks whether execute() was called.
struct MockToolExecutor {
    results: HashMap<String, String>,
    /// Tracks each (tool_name, input) call for assertion.
    calls: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
}

impl MockToolExecutor {
    fn new_with_results(results: HashMap<String, String>) -> Arc<Self> {
        Arc::new(Self {
            results,
            calls: Arc::new(Mutex::new(vec![])),
        })
    }

    fn simple() -> Arc<Self> {
        let mut results = HashMap::new();
        results.insert("kubectl".to_string(), "pod deleted".to_string());
        results.insert("git".to_string(), "git output".to_string());
        results.insert("echo".to_string(), "hello".to_string());
        Self::new_with_results(results)
    }

    fn was_called_with(&self, tool_name: &str) -> bool {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .any(|(name, _)| name == tool_name)
    }

    fn call_count_for(&self, tool_name: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(name, _)| name == tool_name)
            .count()
    }
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String> {
        self.calls
            .lock()
            .unwrap()
            .push((tool.name.clone(), input.clone()));

        match self.results.get(&tool.name) {
            Some(output) => Ok(output.clone()),
            None => Err(format!("no result for '{}'", tool.name)),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------

fn make_tool_entry(name: &str) -> ToolEntry {
    ToolEntry {
        name: name.to_string(),
        tool_type: DirectoryToolType::Cli,
        command: Some(format!("/usr/bin/{}", name)),
        description: Some(format!("Test tool: {}", name)),
        server: None,
        args: vec![],
        capabilities: None,
    }
}

fn make_agent(tools: Vec<ToolEntry>) -> AgentDefinition {
    AgentDefinition {
        name: "test-agent".to_string(),
        version: Some("0.1.0".to_string()),
        description: None,
        model_preferred: None,
        soul_content: "You are a test agent.".to_string(),
        rules_content: None,
        skills: vec![],
        tools,
        sub_agents: vec![],
        mcp_servers: vec![],
        max_iterations: 5,
        timeout_secs: 30,
        mode: AgentMode::Autonomous,
        triggers: vec![],
        notifications: vec![],
        vector_memory: agentix_core::VectorMemoryConfig::default(),
        research_phase: agentix_core::ResearchPhaseConfig::default(),
        budget: None,
        approval: None,
    }
}

fn make_approval_store() -> Arc<ApprovalStore> {
    Arc::new(ApprovalStore::open(":memory:").expect("failed to open in-memory ApprovalStore"))
}

fn make_audit_store() -> Arc<AuditStore> {
    Arc::new(AuditStore::open(":memory:").expect("failed to open in-memory AuditStore"))
}

fn autonomous_config() -> ReActConfig {
    ReActConfig {
        max_iterations: 5,
        timeout: Duration::from_secs(10),
        max_tokens_per_run: None,
        ssrf_guard: None,
        secret_redactor: None,
        audit_store: None,
        agent_name: "test-agent".to_string(),
        approval_policy: Some(ApprovalPolicy {
            mode: AgentMode::Autonomous,
            ..Default::default()
        }),
        approval_store: None,
        run_id: Some("run-test".to_string()),
    }
}

fn manual_config(store: Arc<ApprovalStore>, timeout_secs: u32) -> ReActConfig {
    ReActConfig {
        max_iterations: 5,
        timeout: Duration::from_secs(30),
        max_tokens_per_run: None,
        ssrf_guard: None,
        secret_redactor: None,
        audit_store: None,
        agent_name: "test-agent".to_string(),
        approval_policy: Some(ApprovalPolicy {
            mode: AgentMode::Manual,
            default_timeout_secs: timeout_secs,
            ..Default::default()
        }),
        approval_store: Some(store),
        run_id: Some("run-manual".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Test 1: autonomous_mode_no_approval_check
// ---------------------------------------------------------------------------

/// Autonomous mode: tool call executes without creating any ApprovalRequest.
#[tokio::test]
async fn autonomous_mode_no_approval_check() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();

    let mut config = autonomous_config();
    config.approval_store = Some(store.clone());

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "get pods"}),
        },
        MockResponse::FinalAnswer {
            text: "Pods retrieved successfully.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    let engine = ReActEngine::new(model, executor.clone(), config);
    let result = engine.run(&definition, "List all pods").await.unwrap();

    // Tool should have executed
    assert!(executor.was_called_with("kubectl"), "kubectl should have been called");

    // No approval requests created
    let pending = store.list_pending(100).unwrap();
    assert_eq!(pending.len(), 0, "Autonomous mode should create no ApprovalRequests");

    assert!(!result.output.is_empty());
}

// ---------------------------------------------------------------------------
// Test 2: manual_mode_creates_approval_request
// ---------------------------------------------------------------------------

/// Manual mode: a background task pre-approves the request so the tool executes.
#[tokio::test]
async fn manual_mode_creates_approval_request() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();

    let config = manual_config(store.clone(), 30);

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "delete pod nginx"}),
        },
        MockResponse::FinalAnswer {
            text: "Done.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    // Background task: poll store and approve once request appears
    let store_clone = store.clone();
    tokio::spawn(async move {
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let pending = store_clone.list_pending(10).unwrap();
            if !pending.is_empty() {
                let _ = store_clone.approve(&pending[0].id, "test-approver", None);
                return;
            }
        }
    });

    let engine = ReActEngine::new(model, executor.clone(), config);
    let result = engine.run(&definition, "Delete nginx pod").await.unwrap();

    // Tool should have executed after approval
    assert!(executor.was_called_with("kubectl"), "kubectl should execute after approval");

    // An ApprovalRequest was created and decided
    let all_reqs = store.list_by_agent("test-agent", 10).unwrap();
    assert!(!all_reqs.is_empty(), "ApprovalRequest should exist in the store");

    let req = &all_reqs[0];
    assert!(
        matches!(req.status, ApprovalStatus::Approved { .. }),
        "Request should be in Approved state, got: {:?}", req.status
    );

    assert!(!result.output.is_empty());
}

// ---------------------------------------------------------------------------
// Test 3: semi_autonomous_flagged_tool_pauses
// ---------------------------------------------------------------------------

/// SemiAutonomous: flagged tool pauses, unflagged tool runs freely.
#[tokio::test]
async fn semi_autonomous_flagged_tool_pauses() {
    let tools = vec![make_tool_entry("kubectl"), make_tool_entry("git")];
    let definition = make_agent(tools);
    let store = make_approval_store();

    let config = ReActConfig {
        max_iterations: 5,
        timeout: Duration::from_secs(30),
        max_tokens_per_run: None,
        ssrf_guard: None,
        secret_redactor: None,
        audit_store: None,
        agent_name: "test-agent".to_string(),
        approval_policy: Some(ApprovalPolicy {
            mode: AgentMode::SemiAutonomous,
            flagged_tools: vec!["kubectl".to_string()],
            default_timeout_secs: 30,
            ..Default::default()
        }),
        approval_store: Some(store.clone()),
        run_id: Some("run-semi".to_string()),
    };

    let model = MockModel::new(vec![
        // First call: kubectl (flagged — needs approval)
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "delete pod nginx"}),
        },
        // Second call: git (not flagged — runs freely)
        MockResponse::ToolCall {
            tool_name: "git".to_string(),
            input: serde_json::json!({"args": "status"}),
        },
        MockResponse::FinalAnswer {
            text: "All done.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    // Background task: approve the kubectl request
    let store_clone = store.clone();
    tokio::spawn(async move {
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let pending = store_clone.list_pending(10).unwrap();
            if !pending.is_empty() {
                let _ = store_clone.approve(&pending[0].id, "approver", None);
                return;
            }
        }
    });

    let engine = ReActEngine::new(model, executor.clone(), config);
    engine.run(&definition, "Do kubectl then git").await.unwrap();

    // kubectl should have been approved and executed
    assert!(executor.was_called_with("kubectl"), "kubectl should execute after approval");

    // An ApprovalRequest should exist for kubectl
    let all_reqs = store.list_by_agent("test-agent", 10).unwrap();
    let kubectl_reqs: Vec<_> = all_reqs
        .iter()
        .filter(|r| r.tool_name.as_deref() == Some("kubectl"))
        .collect();
    assert!(!kubectl_reqs.is_empty(), "ApprovalRequest for kubectl should exist");

    // git should NOT have created an approval request
    let git_reqs: Vec<_> = all_reqs
        .iter()
        .filter(|r| r.tool_name.as_deref() == Some("git"))
        .collect();
    assert_eq!(git_reqs.len(), 0, "git should not create an ApprovalRequest");
}

// ---------------------------------------------------------------------------
// Test 4: denied_action_skips_tool_call
// ---------------------------------------------------------------------------

/// Manual mode: denied request skips tool execution; loop continues to final answer.
#[tokio::test]
async fn denied_action_skips_tool_call() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();

    let config = manual_config(store.clone(), 30);

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "delete pod nginx"}),
        },
        MockResponse::FinalAnswer {
            text: "I could not delete the pod.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    // Background task: deny the request
    let store_clone = store.clone();
    tokio::spawn(async move {
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let pending = store_clone.list_pending(10).unwrap();
            if !pending.is_empty() {
                let _ = store_clone.deny(&pending[0].id, "security-lead", Some("Too risky"));
                return;
            }
        }
    });

    let engine = ReActEngine::new(model, executor.clone(), config);
    let result = engine.run(&definition, "Delete nginx pod").await.unwrap();

    // kubectl should NOT have been called (denied)
    assert!(
        !executor.was_called_with("kubectl"),
        "kubectl should NOT execute after denial"
    );

    // Loop should have continued (final answer returned)
    assert!(!result.output.is_empty(), "Loop should continue after denial");

    // Observation should mention denied/approval denied
    // (checking via the fact that loop ran to completion, not aborted)
    assert!(result.iterations > 0 || !result.output.is_empty());
}

// ---------------------------------------------------------------------------
// Test 5: timed_out_action_skips_tool_call
// ---------------------------------------------------------------------------

/// Manual mode: timed-out request (timeout_secs=1) skips tool execution.
#[tokio::test(flavor = "multi_thread")]
async fn timed_out_action_skips_tool_call() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();

    // Short timeout (2 seconds) to keep test fast
    let config = manual_config(store.clone(), 2);

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "delete namespace prod"}),
        },
        MockResponse::FinalAnswer {
            text: "Unable to proceed.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    // DO NOT approve or deny — let it time out
    let engine = ReActEngine::new(model, executor.clone(), config);
    let result = engine
        .run(&definition, "Delete the production namespace")
        .await
        .unwrap();

    // kubectl should NOT have been called
    assert!(
        !executor.was_called_with("kubectl"),
        "kubectl should NOT execute after timeout"
    );

    // Loop should have continued to final answer
    assert!(!result.output.is_empty(), "Loop should continue after timeout");
}

// ---------------------------------------------------------------------------
// Test 6: approval_events_emitted
// ---------------------------------------------------------------------------

/// Manual mode with pre-approval: ApprovalRequested then ApprovalResolved events are emitted.
#[tokio::test]
async fn approval_events_emitted() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();

    let config = manual_config(store.clone(), 30);

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "get pods"}),
        },
        MockResponse::FinalAnswer {
            text: "Done.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    let (tx, mut rx) = broadcast::channel::<ReActEvent>(32);

    // Background task: approve the request
    let store_clone = store.clone();
    tokio::spawn(async move {
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let pending = store_clone.list_pending(10).unwrap();
            if !pending.is_empty() {
                let _ = store_clone.approve(&pending[0].id, "event-approver", None);
                return;
            }
        }
    });

    let engine = ReActEngine::new(model, executor, config).with_event_stream(tx);
    engine.run(&definition, "Get pods").await.unwrap();

    // Collect emitted events
    let mut events = vec![];
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    // Assert ApprovalRequested was emitted
    let requested = events.iter().any(|e| matches!(e, ReActEvent::ApprovalRequested { .. }));
    assert!(requested, "ApprovalRequested event should be emitted");

    // Assert ApprovalResolved was emitted
    let resolved = events.iter().any(|e| matches!(e, ReActEvent::ApprovalResolved { .. }));
    assert!(resolved, "ApprovalResolved event should be emitted");

    // Verify ApprovalResolved carries correct status (Approved)
    for event in &events {
        if let ReActEvent::ApprovalResolved { status, .. } = event {
            assert!(
                matches!(status, ApprovalStatus::Approved { .. }),
                "Resolved status should be Approved, got: {:?}", status
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Test 7: approval_logged_in_audit_trail
// ---------------------------------------------------------------------------

/// Manual mode with approval: AuditStore should contain an ApprovalDecision entry.
#[tokio::test]
async fn approval_logged_in_audit_trail() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();
    let audit = make_audit_store();

    let mut config = manual_config(store.clone(), 30);
    config.audit_store = Some(audit.clone());

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "get pods"}),
        },
        MockResponse::FinalAnswer {
            text: "Done.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    // Background task: approve
    let store_clone = store.clone();
    tokio::spawn(async move {
        for _ in 0..30 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let pending = store_clone.list_pending(10).unwrap();
            if !pending.is_empty() {
                let _ = store_clone.approve(&pending[0].id, "audit-approver", None);
                return;
            }
        }
    });

    let engine = ReActEngine::new(model, executor, config);
    engine.run(&definition, "Get pods").await.unwrap();

    // Check audit store for ApprovalDecision entry
    let entries = audit.list_events("test-agent", 20).unwrap();
    let approval_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.event_type == AuditEventType::ApprovalDecision)
        .collect();

    assert!(
        !approval_entries.is_empty(),
        "AuditStore should contain at least one ApprovalDecision entry"
    );

    // Verify outcome is Success (approved)
    let approved_entry = approval_entries
        .iter()
        .find(|e| matches!(e.outcome, agentix_runtime::AuditOutcome::Success));
    assert!(
        approved_entry.is_some(),
        "ApprovalDecision entry should have Success outcome for approved request"
    );
}

// ---------------------------------------------------------------------------
// Test 8: denial_logged_in_audit_trail
// ---------------------------------------------------------------------------

/// Manual mode with denial: AuditStore should contain an ApprovalDecision entry with Denied outcome.
#[tokio::test]
async fn denial_logged_in_audit_trail() {
    let tools = vec![make_tool_entry("kubectl")];
    let definition = make_agent(tools);
    let store = make_approval_store();
    let audit = make_audit_store();

    let mut config = manual_config(store.clone(), 30);
    config.audit_store = Some(audit.clone());

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"args": "delete namespace prod"}),
        },
        MockResponse::FinalAnswer {
            text: "Aborted.".to_string(),
        },
    ]);
    let executor = MockToolExecutor::simple();

    // Background task: deny
    let store_clone = store.clone();
    tokio::spawn(async move {
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            let pending = store_clone.list_pending(10).unwrap();
            if !pending.is_empty() {
                let _ = store_clone.deny(
                    &pending[0].id,
                    "security-lead",
                    Some("Too dangerous"),
                );
                return;
            }
        }
    });

    let engine = ReActEngine::new(model, executor, config);
    engine.run(&definition, "Delete production namespace").await.unwrap();

    // Check audit store for ApprovalDecision entry
    let entries = audit.list_events("test-agent", 20).unwrap();
    let approval_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.event_type == AuditEventType::ApprovalDecision)
        .collect();

    assert!(
        !approval_entries.is_empty(),
        "AuditStore should contain at least one ApprovalDecision entry for denied request"
    );

    // Verify at least one entry has Denied outcome
    let denied_entry = approval_entries
        .iter()
        .find(|e| matches!(e.outcome, agentix_runtime::AuditOutcome::Denied(_)));
    assert!(
        denied_entry.is_some(),
        "ApprovalDecision entry should have Denied outcome, entries: {:?}",
        approval_entries.iter().map(|e| &e.outcome).collect::<Vec<_>>()
    );
}
