//! TDD tests for the ReAct (Reason + Act) loop engine.
//!
//! These tests use mock implementations of `Model` and `ToolExecutor`
//! to drive the loop without real LLM or tool infrastructure.

use agentix_core::{
    AgentDefinition, AgentMode, AgentixError, AgentixResult, ModelConfig, ModelProvider,
    ModelRequest, ModelResponse, RequestMessage, SkillEntry, StopReason, ToolCall, Usage,
};
use agentix_core::agent::{DirectoryToolType, ToolEntry};
use agentix_runtime::executor::react_loop::{
    ReActConfig, ReActEngine, ReActEvent, RunResult, ToolAction, ToolExecutor,
};

use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

// ---------------------------------------------------------------------------
// Mock infrastructure
// ---------------------------------------------------------------------------

/// Canned response from the mock LLM.
enum MockResponse {
    /// The model requests a tool call.
    ToolCall {
        tool_name: String,
        input: serde_json::Value,
    },
    /// The model produces a final text answer.
    FinalAnswer { text: String },
}

/// Mock LLM that returns pre-programmed responses in order.
struct MockModel {
    responses: Vec<MockResponse>,
    call_count: AtomicUsize,
    /// Capture the system prompt from the first request.
    captured_system_prompt: Mutex<Option<String>>,
}

impl MockModel {
    fn new(responses: Vec<MockResponse>) -> Arc<Self> {
        Arc::new(Self {
            responses,
            call_count: AtomicUsize::new(0),
            captured_system_prompt: Mutex::new(None),
        })
    }
}

#[async_trait]
impl agentix_core::Model for MockModel {
    async fn generate(&self, request: &ModelRequest) -> AgentixResult<ModelResponse> {
        // Capture the system prompt on the first call.
        {
            let mut lock = self.captured_system_prompt.lock().unwrap();
            if lock.is_none() {
                *lock = request.system.clone();
            }
        }

        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);

        if idx >= self.responses.len() {
            // Beyond all scripted responses — return a final answer.
            return Ok(ModelResponse {
                content: "fallback final answer".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage::default(),
                metadata: HashMap::new(),
            });
        }

        match &self.responses[idx] {
            MockResponse::ToolCall { tool_name, input } => Ok(ModelResponse {
                content: format!("Thinking… I will call {}", tool_name),
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
        unimplemented!("streaming not needed for ReAct tests")
    }

    fn config(&self) -> &ModelConfig {
        unimplemented!("config not needed for ReAct tests")
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Mock tool executor with pre-programmed results keyed by tool name.
struct MockToolExecutor {
    /// Maps tool name → Ok(output) | Err(error_msg).
    results: HashMap<String, Result<String, String>>,
    /// Records every (tool_name, input) call.
    calls: Mutex<Vec<(String, serde_json::Value)>>,
}

impl MockToolExecutor {
    fn new(results: HashMap<String, Result<String, String>>) -> Arc<Self> {
        Arc::new(Self {
            results,
            calls: Mutex::new(vec![]),
        })
    }

    fn called_with(&self) -> Vec<(String, serde_json::Value)> {
        self.calls.lock().unwrap().clone()
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
            Some(Ok(output)) => Ok(output.clone()),
            Some(Err(err)) => Err(err.clone()),
            None => Err(format!("MockToolExecutor: no result registered for '{}'", tool.name)),
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
    }
}

fn make_agent_definition(
    soul: &str,
    rules: Option<&str>,
    skills: Vec<(&str, &str)>,
    tools: Vec<ToolEntry>,
    max_iterations: u32,
    timeout_secs: u64,
) -> AgentDefinition {
    AgentDefinition {
        name: "test-agent".to_string(),
        version: Some("0.1.0".to_string()),
        description: None,
        model_preferred: None,
        soul_content: soul.to_string(),
        rules_content: rules.map(|r| r.to_string()),
        skills: skills
            .into_iter()
            .map(|(n, c)| SkillEntry {
                name: n.to_string(),
                content: c.to_string(),
            })
            .collect(),
        tools,
        sub_agents: vec![],
        mcp_servers: vec![],
        max_iterations,
        timeout_secs,
        mode: AgentMode::Autonomous,
        triggers: vec![],
        notifications: vec![],
        vector_memory: agentix_core::VectorMemoryConfig::default(),
        research_phase: agentix_core::ResearchPhaseConfig::default(),
    }
}

fn default_config() -> ReActConfig {
    ReActConfig {
        max_iterations: 10,
        timeout: Duration::from_secs(30),
    }
}

// ---------------------------------------------------------------------------
// Tests — Core loop behaviour
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_react_loop_single_iteration() {
    let tool = make_tool_entry("kubectl");
    let definition = make_agent_definition(
        "You are a helpful Kubernetes operator.",
        None,
        vec![],
        vec![tool.clone()],
        10,
        30,
    );

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "kubectl".to_string(),
            input: serde_json::json!({"command": "get pods"}),
        },
        MockResponse::FinalAnswer {
            text: "The pods are running.".to_string(),
        },
    ]);

    let executor = MockToolExecutor::new({
        let mut m = HashMap::new();
        m.insert("kubectl".to_string(), Ok("NAME   READY   STATUS\npod-1  1/1     Running".to_string()));
        m
    });

    let engine = ReActEngine::new(model, executor, default_config());
    let result = engine.run(&definition, "List all pods").await.unwrap();

    assert_eq!(result.iterations, 1, "Expected 1 iteration (1 tool call + 1 final answer)");
    assert!(!result.output.is_empty(), "Output should be non-empty");
    assert!(!result.reached_max_iterations);
}

#[tokio::test]
async fn test_react_loop_multiple_iterations() {
    let tool = make_tool_entry("shell");
    let definition = make_agent_definition(
        "You are a shell automation agent.",
        None,
        vec![],
        vec![tool.clone()],
        10,
        30,
    );

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "shell".to_string(),
            input: serde_json::json!({"cmd": "ls /tmp"}),
        },
        MockResponse::ToolCall {
            tool_name: "shell".to_string(),
            input: serde_json::json!({"cmd": "cat /tmp/file.txt"}),
        },
        MockResponse::FinalAnswer {
            text: "Done reading the file.".to_string(),
        },
    ]);

    let executor = MockToolExecutor::new({
        let mut m = HashMap::new();
        m.insert("shell".to_string(), Ok("result".to_string()));
        m
    });

    let engine = ReActEngine::new(model, executor, default_config());
    let result = engine.run(&definition, "Read /tmp/file.txt").await.unwrap();

    assert_eq!(result.iterations, 2, "Expected 2 tool-call iterations");
    assert_eq!(result.tool_calls.len(), 2, "Expected 2 tool calls total");
    assert!(!result.reached_max_iterations);
}

#[tokio::test]
async fn test_react_loop_max_iterations_reached() {
    let tool = make_tool_entry("never-done");
    let definition = make_agent_definition(
        "An agent that never finishes.",
        None,
        vec![],
        vec![tool.clone()],
        2, // max_iterations = 2
        30,
    );

    // Always return a tool call — never a final answer.
    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "never-done".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::ToolCall {
            tool_name: "never-done".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::ToolCall {
            tool_name: "never-done".to_string(),
            input: serde_json::json!({}),
        },
    ]);

    let executor = MockToolExecutor::new({
        let mut m = HashMap::new();
        m.insert("never-done".to_string(), Ok("still working".to_string()));
        m
    });

    let config = ReActConfig {
        max_iterations: 2,
        timeout: Duration::from_secs(30),
    };

    let engine = ReActEngine::new(model, executor, config);
    let result = engine.run(&definition, "Do something").await.unwrap();

    assert!(result.reached_max_iterations, "Should have reached max iterations");
    assert_eq!(result.iterations, 2, "Should have run exactly 2 iterations");
}

#[tokio::test]
async fn test_react_loop_no_tool_call() {
    let definition = make_agent_definition(
        "A simple conversational agent.",
        None,
        vec![],
        vec![],
        10,
        30,
    );

    let model = MockModel::new(vec![MockResponse::FinalAnswer {
        text: "42 is the answer.".to_string(),
    }]);

    let executor = MockToolExecutor::new(HashMap::new());

    let engine = ReActEngine::new(model, executor, default_config());
    let result = engine.run(&definition, "What is the answer?").await.unwrap();

    assert_eq!(result.iterations, 1, "Expected 1 iteration (immediate final answer)");
    assert!(result.tool_calls.is_empty(), "No tool calls should have been made");
    assert_eq!(result.output, "42 is the answer.");
    assert!(!result.reached_max_iterations);
}

#[tokio::test]
async fn test_react_loop_tool_failure_feeds_back() {
    let tool = make_tool_entry("failing-tool");
    let definition = make_agent_definition(
        "An agent that handles errors.",
        None,
        vec![],
        vec![tool.clone()],
        10,
        30,
    );

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "failing-tool".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::FinalAnswer {
            text: "I encountered an error but handled it.".to_string(),
        },
    ]);

    let executor = MockToolExecutor::new({
        let mut m = HashMap::new();
        m.insert("failing-tool".to_string(), Err("permission denied".to_string()));
        m
    });

    let engine = ReActEngine::new(model.clone(), executor, default_config());
    let result = engine.run(&definition, "Do the thing").await.unwrap();

    // The loop should NOT hard-fail; it continues after the tool error.
    assert!(!result.reached_max_iterations, "Should have completed, not hit max iterations");
    assert!(
        !result.output.is_empty(),
        "Should have produced output after handling tool failure"
    );

    // Verify the error was fed back: we check by ensuring the model received 2 calls
    // (first call returned tool call, second call should have seen the error observation).
    assert_eq!(
        model.call_count.load(Ordering::SeqCst),
        2,
        "Model should have been called twice: once for tool call, once after error observation"
    );
}

#[tokio::test]
async fn test_react_loop_uses_resolved_system_prompt() {
    let definition = make_agent_definition(
        "SOUL_CONTENT: I am the soul.",
        Some("RULES_CONTENT: Never do bad things."),
        vec![("my-skill", "SKILL_CONTENT: I know how to code.")],
        vec![],
        10,
        30,
    );

    let model = MockModel::new(vec![MockResponse::FinalAnswer {
        text: "Done.".to_string(),
    }]);

    let executor = MockToolExecutor::new(HashMap::new());
    let engine = ReActEngine::new(model.clone(), executor, default_config());

    let _ = engine.run(&definition, "Hello").await.unwrap();

    let captured = model
        .captured_system_prompt
        .lock()
        .unwrap()
        .clone()
        .expect("Model should have received a system prompt");

    assert!(
        captured.contains("SOUL_CONTENT"),
        "System prompt should contain SOUL content; got: {}", captured
    );
    assert!(
        captured.contains("## Constraints"),
        "System prompt should contain ## Constraints header; got: {}", captured
    );
    assert!(
        captured.contains("RULES_CONTENT"),
        "System prompt should contain RULES content; got: {}", captured
    );
    assert!(
        captured.contains("## Skill:"),
        "System prompt should contain ## Skill: header; got: {}", captured
    );
    assert!(
        captured.contains("SKILL_CONTENT"),
        "System prompt should contain skill content; got: {}", captured
    );
}

#[tokio::test]
async fn test_react_loop_uses_agent_definition_tools() {
    let tool_a = make_tool_entry("tool-alpha");
    let tool_b = make_tool_entry("tool-beta");

    let definition = make_agent_definition(
        "A two-tool agent.",
        None,
        vec![],
        vec![tool_a.clone(), tool_b.clone()],
        10,
        30,
    );

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "tool-beta".to_string(),
            input: serde_json::json!({"key": "value"}),
        },
        MockResponse::FinalAnswer {
            text: "Used tool-beta.".to_string(),
        },
    ]);

    let executor = MockToolExecutor::new({
        let mut m = HashMap::new();
        m.insert("tool-alpha".to_string(), Ok("alpha output".to_string()));
        m.insert("tool-beta".to_string(), Ok("beta output".to_string()));
        m
    });

    let exec_clone = Arc::clone(&executor);
    let engine = ReActEngine::new(model, executor, default_config());
    let _ = engine.run(&definition, "Use tool-beta please").await.unwrap();

    let calls = exec_clone.called_with();
    assert_eq!(calls.len(), 1, "Expected exactly one tool call");
    assert_eq!(calls[0].0, "tool-beta", "Expected tool-beta to be called");
}

// ---------------------------------------------------------------------------
// Tests — Event emission
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_react_step_events_emitted() {
    let tool = make_tool_entry("event-tool");
    let definition = make_agent_definition(
        "Event emission agent.",
        None,
        vec![],
        vec![tool],
        10,
        30,
    );

    let model = MockModel::new(vec![
        MockResponse::ToolCall {
            tool_name: "event-tool".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::ToolCall {
            tool_name: "event-tool".to_string(),
            input: serde_json::json!({}),
        },
        MockResponse::FinalAnswer {
            text: "All events emitted.".to_string(),
        },
    ]);

    let executor = MockToolExecutor::new({
        let mut m = HashMap::new();
        m.insert("event-tool".to_string(), Ok("event-tool output".to_string()));
        m
    });

    let (tx, mut rx) = broadcast::channel::<ReActEvent>(32);

    let engine = ReActEngine::new(model, executor, default_config()).with_event_stream(tx);
    let result = engine.run(&definition, "Emit events please").await.unwrap();

    // Collect all events.
    let mut events = vec![];
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    assert_eq!(result.iterations, 2, "Expected 2 iterations");

    let step_count = events.iter().filter(|e| matches!(e, ReActEvent::Step(_))).count();
    let complete_count = events
        .iter()
        .filter(|e| matches!(e, ReActEvent::Complete(_)))
        .count();

    assert_eq!(step_count, 2, "Expected 2 ReActEvent::Step events");
    assert_eq!(complete_count, 1, "Expected 1 ReActEvent::Complete event");

    // Verify Step events have expected fields populated.
    for event in &events {
        if let ReActEvent::Step(step) = event {
            assert!(
                !step.plan.is_empty(),
                "Step plan should be non-empty"
            );
            assert!(
                !step.observation.is_empty(),
                "Step observation should be non-empty"
            );
        }
    }
}

#[tokio::test]
async fn test_react_loop_emits_completion_event() {
    let definition = make_agent_definition(
        "Completion event agent.",
        None,
        vec![],
        vec![],
        10,
        30,
    );

    let model = MockModel::new(vec![MockResponse::FinalAnswer {
        text: "Final output text.".to_string(),
    }]);

    let executor = MockToolExecutor::new(HashMap::new());

    let (tx, mut rx) = broadcast::channel::<ReActEvent>(16);

    let engine = ReActEngine::new(model, executor, default_config()).with_event_stream(tx);
    let result = engine.run(&definition, "Finish quickly").await.unwrap();

    // Collect events.
    let mut events = vec![];
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    let last = events.last().expect("At least one event should have been emitted");
    assert!(
        matches!(last, ReActEvent::Complete(_)),
        "Last event should be ReActEvent::Complete"
    );

    if let ReActEvent::Complete(run) = last {
        assert_eq!(run.output, result.output, "Complete event output should match RunResult output");
    }
}
