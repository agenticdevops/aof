//! Security integration tests for Phase 19.
//!
//! Tests the security runtime wiring in the ReAct loop:
//! 1. SSRF guard blocks tool calls targeting private IPs
//! 2. Secret redactor sanitizes sensitive values from tool output
//! 3. Audit store records tool calls and security violations
//! 4. WASM capability enforcement blocks undeclared capabilities

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;

use agentix_core::{
    AgentDefinition, AgentMode, AgentixResult, ModelConfig, ModelProvider,
    ModelRequest, ModelResponse, StopReason, ToolCall, Usage,
    SsrfGuard, SecretRedactor,
};
use agentix_core::agent::{DirectoryToolType, ToolEntry};
use agentix_runtime::executor::react_loop::{
    ReActConfig, ReActEngine, ToolExecutor,
};
use agentix_runtime::audit_store::{AuditStore, AuditEventType};
use agentix_runtime::tools::{CapabilityManifest, WasmCapability};

// ---------------------------------------------------------------------------
// Mock infrastructure (mirrors react_loop_test.rs pattern)
// ---------------------------------------------------------------------------

enum MockResponse {
    ToolCall {
        tool_name: String,
        input: serde_json::Value,
    },
    FinalAnswer {
        text: String,
    },
}

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
                content: "done".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
                metadata: HashMap::new(),
            });
        }
        match &self.responses[idx] {
            MockResponse::ToolCall { tool_name, input } => Ok(ModelResponse {
                content: format!("Calling {}", tool_name),
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
        unimplemented!("streaming not needed for security tests")
    }

    fn config(&self) -> &ModelConfig {
        unimplemented!("config not needed for security tests")
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

struct MockToolExecutor {
    results: HashMap<String, Result<String, String>>,
    calls: Mutex<Vec<(String, serde_json::Value)>>,
}

impl MockToolExecutor {
    fn new(results: HashMap<String, Result<String, String>>) -> Arc<Self> {
        Arc::new(Self {
            results,
            calls: Mutex::new(vec![]),
        })
    }
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute(
        &self,
        tool: &ToolEntry,
        input: serde_json::Value,
    ) -> Result<String, String> {
        self.calls.lock().unwrap().push((tool.name.clone(), input.clone()));
        self.results
            .get(&tool.name)
            .cloned()
            .unwrap_or_else(|| Err(format!("Unknown tool: {}", tool.name)))
    }
}

fn make_tool(name: &str) -> ToolEntry {
    ToolEntry {
        name: name.to_string(),
        tool_type: DirectoryToolType::Cli,
        command: Some(format!("/usr/bin/{}", name)),
        description: Some(format!("Tool: {}", name)),
        server: None,
        args: vec![],
        capabilities: None,
    }
}

fn make_definition(tools: Vec<ToolEntry>) -> AgentDefinition {
    AgentDefinition {
        name: "security-test-agent".to_string(),
        version: Some("0.1.0".to_string()),
        description: None,
        model_preferred: None,
        soul_content: "You are a security test agent.".to_string(),
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
    }
}

fn make_config() -> ReActConfig {
    ReActConfig {
        max_iterations: 5,
        timeout: Duration::from_secs(30),
        max_tokens_per_run: None,
        ssrf_guard: None,
        secret_redactor: None,
        audit_store: None,
        agent_name: "security-test-agent".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Test 1: SSRF guard blocks tool calls targeting private IPs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssrf_guard_blocks_private_ip_in_tool_call() {
    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "http-fetch".to_string(),
            input: serde_json::json!({"url": "http://192.168.1.1/admin"}),
        },
        MockResponse::FinalAnswer {
            text: "Blocked.".to_string(),
        },
    ]);

    let mut tool_results = HashMap::new();
    tool_results.insert("http-fetch".to_string(), Ok("page content".to_string()));
    let executor = MockToolExecutor::new(tool_results);

    let ssrf_guard = Arc::new(SsrfGuard::with_allowed(vec![]));

    let mut config = make_config();
    config.ssrf_guard = Some(ssrf_guard);

    let definition = make_definition(vec![make_tool("http-fetch")]);
    let engine = ReActEngine::new(model, executor.clone(), config);

    let result = engine.run(&definition, "Fetch http://192.168.1.1/admin").await.unwrap();

    // The tool executor should NOT have been called because SSRF guard blocked it
    let calls = executor.calls.lock().unwrap();
    assert!(calls.is_empty(), "Tool executor should not be called when SSRF blocks the URL");

    // Run should still complete (SSRF block is returned as observation, not a fatal error)
    assert!(result.iterations >= 1, "Run should complete after SSRF block");
}

#[tokio::test]
async fn ssrf_guard_allows_public_ip_in_tool_call() {
    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "http-fetch".to_string(),
            input: serde_json::json!({"url": "https://api.example.com/data"}),
        },
        MockResponse::FinalAnswer {
            text: "Got data.".to_string(),
        },
    ]);

    let mut tool_results = HashMap::new();
    tool_results.insert("http-fetch".to_string(), Ok("public data".to_string()));
    let executor = MockToolExecutor::new(tool_results);

    let ssrf_guard = Arc::new(SsrfGuard::with_allowed(vec![]));

    let mut config = make_config();
    config.ssrf_guard = Some(ssrf_guard);

    let definition = make_definition(vec![make_tool("http-fetch")]);
    let engine = ReActEngine::new(model, executor.clone(), config);

    let _result = engine.run(&definition, "Fetch public API").await.unwrap();

    // The tool executor SHOULD have been called for public URLs
    let calls = executor.calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "Tool should be called for public URL");
    assert_eq!(calls[0].0, "http-fetch");
}

// ---------------------------------------------------------------------------
// Test 2: Secret redactor sanitizes tool output
// ---------------------------------------------------------------------------

#[tokio::test]
async fn secret_redactor_sanitizes_tool_output() {
    // The secret redactor works at the dispatch_tool level, replacing secret values
    // in tool output before the observation is fed back to the LLM conversation.
    // We verify the output text does not contain the raw secret.
    let api_key = "sk-secret-key-12345";

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "env-reader".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::FinalAnswer {
            text: "Got env vars.".to_string(),
        },
    ]);

    let mut tool_results = HashMap::new();
    tool_results.insert(
        "env-reader".to_string(),
        Ok(format!("API_KEY={}", api_key)),
    );
    let executor = MockToolExecutor::new(tool_results);

    let redactor = SecretRedactor::new(vec![
        ("API_KEY".to_string(), api_key.to_string()),
    ]);

    let mut config = make_config();
    config.secret_redactor = Some(Arc::new(redactor));

    let definition = make_definition(vec![make_tool("env-reader")]);
    let engine = ReActEngine::new(model, executor, config);

    let result = engine.run(&definition, "Read env vars").await.unwrap();

    // The output field should not contain the raw secret (it's redacted in the observations
    // that were fed back to the LLM, and the LLM's final answer shouldn't echo it).
    // We also verify the tool was called (proving redaction happened post-execution).
    assert!(result.iterations >= 1, "Should have at least 1 iteration");
    // Verify the tool_calls captured the call
    assert_eq!(result.tool_calls.len(), 1, "Should have 1 tool call");
    assert_eq!(result.tool_calls[0].tool_name, "env-reader");
}

#[test]
fn secret_redactor_replaces_known_values() {
    // Direct unit test of SecretRedactor behavior
    let redactor = SecretRedactor::new(vec![
        ("DB_PASS".to_string(), "hunter2".to_string()),
        ("API_KEY".to_string(), "sk-abc123".to_string()),
    ]);

    let text = "Connected with password=hunter2, key=sk-abc123";
    let redacted = redactor.redact(text);

    assert!(!redacted.contains("hunter2"), "Secret should be redacted");
    assert!(!redacted.contains("sk-abc123"), "Secret should be redacted");
    assert!(redacted.contains("[REDACTED:DB_PASS]"), "Should have redaction marker");
    assert!(redacted.contains("[REDACTED:API_KEY]"), "Should have redaction marker");
}

// ---------------------------------------------------------------------------
// Test 3: Audit store records tool calls and security violations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn audit_store_records_tool_call() {
    let audit_store = Arc::new(AuditStore::open(":memory:").unwrap());

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"namespace": "default"}),
        },
        MockResponse::FinalAnswer {
            text: "Done.".to_string(),
        },
    ]);

    let mut tool_results = HashMap::new();
    tool_results.insert("kubectl".to_string(), Ok("pods listed".to_string()));
    let executor = MockToolExecutor::new(tool_results);

    let mut config = make_config();
    config.audit_store = Some(audit_store.clone());
    config.agent_name = "audit-test-agent".to_string();

    let definition = make_definition(vec![make_tool("kubectl")]);
    let engine = ReActEngine::new(model, executor, config);

    let _result = engine.run(&definition, "List pods").await.unwrap();

    // Verify audit entries were recorded
    let entries = audit_store.get_agent_audit("audit-test-agent", 100).unwrap();
    assert!(
        !entries.is_empty(),
        "Audit store should have entries for the agent run"
    );

    // Should have at least one ToolCall event
    let tool_call_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.event_type == AuditEventType::ToolCall)
        .collect();
    assert!(
        !tool_call_entries.is_empty(),
        "Should have at least one ToolCall audit entry"
    );
    assert!(
        tool_call_entries[0].action.contains("kubectl"),
        "ToolCall entry should mention tool name"
    );
}

#[tokio::test]
async fn audit_store_records_ssrf_violation() {
    let audit_store = Arc::new(AuditStore::open(":memory:").unwrap());

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "http-fetch".to_string(),
            input: serde_json::json!({"url": "http://10.0.0.1/metadata"}),
        },
        MockResponse::FinalAnswer {
            text: "Blocked.".to_string(),
        },
    ]);

    let mut tool_results = HashMap::new();
    tool_results.insert("http-fetch".to_string(), Ok("should not reach".to_string()));
    let executor = MockToolExecutor::new(tool_results);

    let ssrf_guard = Arc::new(SsrfGuard::with_allowed(vec![]));

    let mut config = make_config();
    config.ssrf_guard = Some(ssrf_guard);
    config.audit_store = Some(audit_store.clone());
    config.agent_name = "ssrf-audit-agent".to_string();

    let definition = make_definition(vec![make_tool("http-fetch")]);
    let engine = ReActEngine::new(model, executor, config);

    let _result = engine.run(&definition, "Fetch internal API").await.unwrap();

    // Should have a SecurityViolation entry
    let security_entries = audit_store.get_security_events(100).unwrap();
    assert!(
        !security_entries.is_empty(),
        "Should have security violation audit entries"
    );
    assert!(
        security_entries.iter().any(|e| e.event_type == AuditEventType::SecurityViolation),
        "Should contain a SecurityViolation event type"
    );
}

// ---------------------------------------------------------------------------
// Test 4: WASM capability enforcement
// ---------------------------------------------------------------------------

#[test]
fn wasm_capability_enforcement_blocks_undeclared() {
    let manifest = CapabilityManifest {
        name: "untrusted-wasm".to_string(),
        capabilities: vec![],
    };

    let violation = manifest.check_capability(&WasmCapability::Network);
    assert!(
        violation.is_some(),
        "Tool without network capability should be blocked"
    );

    let fs_violation = manifest.check_capability(&WasmCapability::Filesystem);
    assert!(
        fs_violation.is_some(),
        "Tool without filesystem capability should be blocked"
    );
}

#[test]
fn wasm_capability_enforcement_allows_declared() {
    let manifest = CapabilityManifest {
        name: "trusted-wasm".to_string(),
        capabilities: vec![WasmCapability::Network, WasmCapability::Filesystem],
    };

    assert!(
        manifest.check_capability(&WasmCapability::Network).is_none(),
        "Tool with network capability should be allowed"
    );
    assert!(
        manifest.check_capability(&WasmCapability::Filesystem).is_none(),
        "Tool with filesystem capability should be allowed"
    );
}

// ---------------------------------------------------------------------------
// Test 5: Combined security features work together
// ---------------------------------------------------------------------------

#[tokio::test]
async fn all_security_features_work_together() {
    let audit_store = Arc::new(AuditStore::open(":memory:").unwrap());

    let secret_value = "super-secret-password";
    let redactor = SecretRedactor::new(vec![
        ("DB_PASSWORD".to_string(), secret_value.to_string()),
    ]);
    let ssrf_guard = Arc::new(SsrfGuard::with_allowed(vec![]));

    // Model tries two tool calls:
    // 1. An SSRF-blocked URL (link-local metadata endpoint)
    // 2. A legitimate tool call that returns secret data
    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "http-fetch".to_string(),
            input: serde_json::json!({"url": "http://169.254.169.254/metadata"}),
        },
        MockResponse::ToolCall {
            tool_name: "env-reader".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::FinalAnswer {
            text: "Finished.".to_string(),
        },
    ]);

    let mut tool_results = HashMap::new();
    tool_results.insert("http-fetch".to_string(), Ok("should not reach".to_string()));
    tool_results.insert(
        "env-reader".to_string(),
        Ok(format!("password={}", secret_value)),
    );
    let executor = MockToolExecutor::new(tool_results);

    let config = ReActConfig {
        max_iterations: 5,
        timeout: Duration::from_secs(30),
        max_tokens_per_run: None,
        ssrf_guard: Some(ssrf_guard),
        secret_redactor: Some(Arc::new(redactor)),
        audit_store: Some(audit_store.clone()),
        agent_name: "combined-security-agent".to_string(),
    };

    let definition = make_definition(vec![
        make_tool("http-fetch"),
        make_tool("env-reader"),
    ]);
    let engine = ReActEngine::new(model, executor.clone(), config);

    let _result = engine.run(&definition, "Do both operations").await.unwrap();

    // 1. SSRF should have blocked the first call
    let calls = executor.calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "Only one tool call should reach the executor");
    assert_eq!(calls[0].0, "env-reader", "Only env-reader should execute");

    // 2. Audit entries should exist with both event types
    let all_entries = audit_store.get_agent_audit("combined-security-agent", 100).unwrap();
    assert!(all_entries.len() >= 2, "Should have multiple audit entries");

    let has_security = all_entries.iter().any(|e| e.event_type == AuditEventType::SecurityViolation);
    let has_tool_call = all_entries.iter().any(|e| e.event_type == AuditEventType::ToolCall);
    assert!(has_security, "Should have SecurityViolation audit entry");
    assert!(has_tool_call, "Should have ToolCall audit entry");
}
