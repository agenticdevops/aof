//! Unit tests for aof-runtime executor

use aof_core::{
    AgentConfig, AgentContext, AofResult, Model, ModelConfig, ModelProvider, ModelRequest,
    ModelResponse, RequestMessage, StopReason, StreamChunk, ToolCall, ToolDefinition,
    ToolExecutor, ToolInput, ToolResult, Usage,
};
use aof_runtime::executor::AgentExecutor;
use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Mock model for testing
struct MockModel {
    responses: Vec<ModelResponse>,
    current: Mutex<usize>,
    config: ModelConfig,
}

impl MockModel {
    fn new(responses: Vec<ModelResponse>) -> Self {
        Self {
            responses,
            current: Mutex::new(0),
            config: ModelConfig {
                model: "mock-model".to_string(),
                provider: ModelProvider::Custom,
                api_key: None,
                endpoint: None,
                temperature: 0.7,
                max_tokens: None,
                timeout_secs: 60,
                headers: HashMap::new(),
                extra: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Model for MockModel {
    async fn generate(&self, _request: &ModelRequest) -> AofResult<ModelResponse> {
        let mut current = self.current.lock().unwrap();
        let idx = *current;
        *current += 1;

        if idx < self.responses.len() {
            Ok(self.responses[idx].clone())
        } else {
            Ok(ModelResponse {
                content: "Done".to_string(),
                tool_calls: vec![],
                stop_reason: StopReason::EndTurn,
                usage: Usage {
                    input_tokens: 10,
                    output_tokens: 5,
                },
                metadata: HashMap::new(),
            })
        }
    }

    async fn generate_stream(
        &self,
        _request: &ModelRequest,
    ) -> AofResult<Pin<Box<dyn futures::Stream<Item = AofResult<StreamChunk>> + Send>>> {
        unimplemented!("Stream not implemented in mock")
    }

    fn config(&self) -> &ModelConfig {
        &self.config
    }

    fn provider(&self) -> ModelProvider {
        ModelProvider::Custom
    }
}

/// Mock tool executor
struct MockToolExecutor {
    should_fail: bool,
}

#[async_trait]
impl ToolExecutor for MockToolExecutor {
    async fn execute_tool(&self, name: &str, _input: ToolInput) -> AofResult<ToolResult> {
        if self.should_fail {
            return Ok(ToolResult::error(format!("Tool {} failed", name)));
        }

        Ok(ToolResult::success(serde_json::json!({
            "tool": name,
            "result": "success"
        })).with_execution_time(50))
    }

    fn list_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters: serde_json::json!({}),
            },
        ]
    }

    fn get_tool(&self, _name: &str) -> Option<Arc<dyn aof_core::Tool>> {
        None
    }
}

#[tokio::test]
async fn test_executor_simple_execution() {
    let config = AgentConfig {
        name: "test-agent".to_string(),
        system_prompt: Some("You are a test assistant".to_string()),
        model: "test-model".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: Some(1000),
        extra: HashMap::new(),
    };

    let responses = vec![ModelResponse {
        content: "Hello! How can I help you?".to_string(),
        tool_calls: vec![],
        stop_reason: StopReason::EndTurn,
        usage: Usage {
            input_tokens: 50,
            output_tokens: 10,
        },
        metadata: HashMap::new(),
    }];

    let model = Box::new(MockModel::new(responses));
    let executor = AgentExecutor::new(config, model, None, None);

    let mut context = AgentContext::new("Hello");
    let result = executor.execute(&mut context).await.unwrap();

    assert_eq!(result, "Hello! How can I help you?");
    assert_eq!(context.metadata.input_tokens, 50);
    assert_eq!(context.metadata.output_tokens, 10);
}

#[tokio::test]
async fn test_executor_with_tool_calls() {
    let config = AgentConfig {
        name: "tool-agent".to_string(),
        system_prompt: None,
        model: "test-model".to_string(),
        tools: vec!["test_tool".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let responses = vec![
        // First response: request tool call
        ModelResponse {
            content: "Let me use a tool".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_1".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({"input": "test"}),
            }],
            stop_reason: StopReason::ToolUse,
            usage: Usage {
                input_tokens: 100,
                output_tokens: 20,
            },
            metadata: HashMap::new(),
        },
        // Second response: final answer
        ModelResponse {
            content: "Here's the result from the tool".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage {
                input_tokens: 150,
                output_tokens: 30,
            },
            metadata: HashMap::new(),
        },
    ];

    let model = Box::new(MockModel::new(responses));
    let tool_executor = Arc::new(MockToolExecutor { should_fail: false });

    let executor = AgentExecutor::new(config, model, Some(tool_executor), None);

    let mut context = AgentContext::new("Do something with a tool");
    let result = executor.execute(&mut context).await.unwrap();

    assert_eq!(result, "Here's the result from the tool");
    assert_eq!(context.metadata.tool_calls, 1);
    assert_eq!(context.tool_results.len(), 1);
}

#[tokio::test]
async fn test_executor_max_iterations() {
    let config = AgentConfig {
        name: "limited-agent".to_string(),
        system_prompt: None,
        model: "test-model".to_string(),
        tools: vec!["test_tool".to_string()],
        memory: None,
        max_iterations: 2,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    // Create responses that always request tools (infinite loop)
    let responses = vec![
        ModelResponse {
            content: "Tool call 1".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_1".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({}),
            }],
            stop_reason: StopReason::ToolUse,
            usage: Usage::default(),
            metadata: HashMap::new(),
        },
        ModelResponse {
            content: "Tool call 2".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_2".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({}),
            }],
            stop_reason: StopReason::ToolUse,
            usage: Usage::default(),
            metadata: HashMap::new(),
        },
        ModelResponse {
            content: "Tool call 3".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_3".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({}),
            }],
            stop_reason: StopReason::ToolUse,
            usage: Usage::default(),
            metadata: HashMap::new(),
        },
    ];

    let model = Box::new(MockModel::new(responses));
    let tool_executor = Arc::new(MockToolExecutor { should_fail: false });

    let executor = AgentExecutor::new(config, model, Some(tool_executor), None);

    let mut context = AgentContext::new("Keep calling tools");
    let result = executor.execute(&mut context).await;

    // Should fail due to max iterations
    assert!(result.is_err());
}

#[tokio::test]
async fn test_executor_tool_failure() {
    let config = AgentConfig {
        name: "failing-tool-agent".to_string(),
        system_prompt: None,
        model: "test-model".to_string(),
        tools: vec!["test_tool".to_string()],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let responses = vec![
        ModelResponse {
            content: "Using tool".to_string(),
            tool_calls: vec![ToolCall {
                id: "call_1".to_string(),
                name: "test_tool".to_string(),
                arguments: serde_json::json!({}),
            }],
            stop_reason: StopReason::ToolUse,
            usage: Usage::default(),
            metadata: HashMap::new(),
        },
        ModelResponse {
            content: "Tool failed, but I'll continue".to_string(),
            tool_calls: vec![],
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
            metadata: HashMap::new(),
        },
    ];

    let model = Box::new(MockModel::new(responses));
    let tool_executor = Arc::new(MockToolExecutor { should_fail: true });

    let executor = AgentExecutor::new(config, model, Some(tool_executor), None);

    let mut context = AgentContext::new("Test tool failure");
    let result = executor.execute(&mut context).await.unwrap();

    assert_eq!(result, "Tool failed, but I'll continue");
    assert_eq!(context.tool_results.len(), 1);
    assert!(!context.tool_results[0].success);
}

#[tokio::test]
async fn test_executor_stop_reasons() {
    let test_cases = vec![
        (StopReason::EndTurn, "Normal completion"),
        (StopReason::MaxTokens, "Max tokens reached"),
        (StopReason::StopSequence, "Stop sequence hit"),
    ];

    for (stop_reason, expected_content) in test_cases {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            system_prompt: None,
            model: "test-model".to_string(),
            tools: vec![],
            memory: None,
            max_iterations: 10,
            temperature: 0.7,
            max_tokens: None,
            extra: HashMap::new(),
        };

        let responses = vec![ModelResponse {
            content: expected_content.to_string(),
            tool_calls: vec![],
            stop_reason,
            usage: Usage::default(),
            metadata: HashMap::new(),
        }];

        let model = Box::new(MockModel::new(responses));
        let executor = AgentExecutor::new(config, model, None, None);

        let mut context = AgentContext::new("Test");
        let result = executor.execute(&mut context).await.unwrap();

        assert_eq!(result, expected_content);
    }
}

#[tokio::test]
async fn test_executor_content_filter() {
    let config = AgentConfig {
        name: "filtered-agent".to_string(),
        system_prompt: None,
        model: "test-model".to_string(),
        tools: vec![],
        memory: None,
        max_iterations: 10,
        temperature: 0.7,
        max_tokens: None,
        extra: HashMap::new(),
    };

    let responses = vec![ModelResponse {
        content: "Inappropriate content".to_string(),
        tool_calls: vec![],
        stop_reason: StopReason::ContentFilter,
        usage: Usage::default(),
        metadata: HashMap::new(),
    }];

    let model = Box::new(MockModel::new(responses));
    let executor = AgentExecutor::new(config, model, None, None);

    let mut context = AgentContext::new("Test content filter");
    let result = executor.execute(&mut context).await;

    assert!(result.is_err());
}
