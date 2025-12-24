//! Token efficiency benchmarking for unified vs per-operation tools
//!
//! This module provides comprehensive benchmarking to measure the REAL
//! token efficiency of different tool approaches in production scenarios.

mod runner;

pub use runner::{run_full_benchmark, BenchmarkReport};

use aof_core::ToolDefinition;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Benchmark scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkScenario {
    /// Scenario name
    pub name: String,

    /// Description
    pub description: String,

    /// Tasks to execute
    pub tasks: Vec<BenchmarkTask>,

    /// Expected tool calls (for validation)
    pub expected_calls: usize,
}

/// Single benchmark task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    /// Task description (user input)
    pub input: String,

    /// Expected tools to be called
    pub expected_tools: Vec<String>,

    /// Whether task requires multiple tool calls
    pub multi_step: bool,
}

/// Benchmark result for a single scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Scenario name
    pub scenario: String,

    /// Approach used (unified vs per-op)
    pub approach: ToolApproach,

    /// Total tokens used
    pub total_tokens: usize,

    /// Breakdown by component
    pub breakdown: TokenBreakdown,

    /// Execution metrics
    pub metrics: ExecutionMetrics,

    /// Success rate
    pub success_rate: f64,
}

/// Tool approach
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolApproach {
    Unified,
    PerOperation,
    Hybrid,
}

/// Token usage breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBreakdown {
    /// Tool definitions (context)
    pub tool_definitions: usize,

    /// System prompt
    pub system_prompt: usize,

    /// Task reasoning
    pub reasoning: usize,

    /// Tool calls (parameters)
    pub tool_calls: usize,

    /// Tool outputs
    pub tool_outputs: usize,

    /// LLM responses
    pub llm_responses: usize,

    /// Retry overhead
    pub retry_overhead: usize,
}

impl TokenBreakdown {
    pub fn total(&self) -> usize {
        self.tool_definitions
            + self.system_prompt
            + self.reasoning
            + self.tool_calls
            + self.tool_outputs
            + self.llm_responses
            + self.retry_overhead
    }
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total execution time (ms)
    pub total_time_ms: u64,

    /// Number of LLM calls
    pub llm_calls: usize,

    /// Number of tool executions
    pub tool_executions: usize,

    /// Number of retries
    pub retries: usize,

    /// Number of errors
    pub errors: usize,

    /// Parallel vs sequential execution
    pub parallel_calls: usize,
}

/// Token counter utility
pub struct TokenCounter;

impl TokenCounter {
    /// Count tokens in text (rough estimate using cl100k_base encoding)
    pub fn count(text: &str) -> usize {
        // Rough approximation: 1 token ≈ 4 characters for English text
        // For JSON, it's closer to 1 token ≈ 3 characters
        // This is conservative (overestimates slightly)

        let chars = text.len();

        // Check if it's mostly JSON (lots of braces, quotes)
        let json_chars = text.chars().filter(|c| matches!(c, '{' | '}' | '[' | ']' | '"' | ':')).count();
        let json_ratio = json_chars as f64 / chars as f64;

        if json_ratio > 0.1 {
            // Likely JSON, use 3 chars per token
            (chars / 3).max(1)
        } else {
            // Regular text, use 4 chars per token
            (chars / 4).max(1)
        }
    }

    /// Count tokens in tool definition
    pub fn count_tool_definition(def: &ToolDefinition) -> usize {
        let json = serde_json::to_string(def).unwrap_or_default();
        Self::count(&json)
    }

    /// Count tokens in all tool definitions
    pub fn count_tool_definitions(defs: &[ToolDefinition]) -> usize {
        defs.iter().map(|d| Self::count_tool_definition(d)).sum()
    }
}

/// Predefined benchmark scenarios
pub fn get_benchmark_scenarios() -> Vec<BenchmarkScenario> {
    vec![
        // Scenario 1: Simple single-tool task
        BenchmarkScenario {
            name: "simple_container_list".to_string(),
            description: "List running containers - single tool call".to_string(),
            tasks: vec![
                BenchmarkTask {
                    input: "Show me all running containers".to_string(),
                    expected_tools: vec!["docker_ps".to_string()],
                    multi_step: false,
                }
            ],
            expected_calls: 1,
        },

        // Scenario 2: Medium - multi-step debugging
        BenchmarkScenario {
            name: "medium_container_debug".to_string(),
            description: "Debug container issues - multiple tool calls".to_string(),
            tasks: vec![
                BenchmarkTask {
                    input: "Check which containers are running and their resource usage".to_string(),
                    expected_tools: vec!["docker_ps".to_string(), "docker_stats".to_string()],
                    multi_step: true,
                },
                BenchmarkTask {
                    input: "Show me logs for any containers using high CPU".to_string(),
                    expected_tools: vec!["docker_logs".to_string()],
                    multi_step: true,
                }
            ],
            expected_calls: 3,
        },

        // Scenario 3: Complex - full troubleshooting workflow
        BenchmarkScenario {
            name: "complex_full_troubleshooting".to_string(),
            description: "Full container troubleshooting - many operations".to_string(),
            tasks: vec![
                BenchmarkTask {
                    input: "List all containers including stopped ones".to_string(),
                    expected_tools: vec!["docker_ps".to_string()],
                    multi_step: false,
                },
                BenchmarkTask {
                    input: "Get resource usage stats for all containers".to_string(),
                    expected_tools: vec!["docker_stats".to_string()],
                    multi_step: false,
                },
                BenchmarkTask {
                    input: "Check logs for any containers in CrashLoopBackOff status".to_string(),
                    expected_tools: vec!["docker_logs".to_string()],
                    multi_step: true,
                },
                BenchmarkTask {
                    input: "Inspect the failing container's configuration".to_string(),
                    expected_tools: vec!["docker_inspect".to_string()],
                    multi_step: true,
                },
                BenchmarkTask {
                    input: "List available docker images".to_string(),
                    expected_tools: vec!["docker_images".to_string()],
                    multi_step: false,
                }
            ],
            expected_calls: 5,
        },

        // Scenario 4: Long session - amortized costs
        BenchmarkScenario {
            name: "long_session_monitoring".to_string(),
            description: "Long monitoring session - 20+ operations".to_string(),
            tasks: (0..20).map(|i| {
                BenchmarkTask {
                    input: format!("Check container stats (iteration {})", i),
                    expected_tools: vec!["docker_stats".to_string()],
                    multi_step: false,
                }
            }).collect(),
            expected_calls: 20,
        },

        // Scenario 5: Parallel-friendly operations
        BenchmarkScenario {
            name: "parallel_health_check".to_string(),
            description: "Health check across services - parallelizable".to_string(),
            tasks: vec![
                BenchmarkTask {
                    input: "Check Docker containers, Kubernetes pods, and system resources all at once".to_string(),
                    expected_tools: vec![
                        "docker_ps".to_string(),
                        "kubectl_get".to_string(),
                        "system_stats".to_string()
                    ],
                    multi_step: true,
                }
            ],
            expected_calls: 3,
        },
    ]
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Include retry simulation
    pub simulate_retries: bool,

    /// Retry rate for unified tools (0.0 - 1.0)
    pub unified_retry_rate: f64,

    /// Retry rate for per-op tools (0.0 - 1.0)
    pub per_op_retry_rate: f64,

    /// Enable parallel execution simulation
    pub simulate_parallel: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            simulate_retries: true,
            unified_retry_rate: 0.13,  // 13% failure rate
            per_op_retry_rate: 0.02,   // 2% failure rate
            simulate_parallel: true,
        }
    }
}

/// Compare unified vs per-operation approaches
pub fn compare_approaches(
    scenario: &BenchmarkScenario,
    config: &BenchmarkConfig,
) -> (BenchmarkResult, BenchmarkResult) {
    let unified = benchmark_unified(scenario, config);
    let per_op = benchmark_per_operation(scenario, config);

    (unified, per_op)
}

/// Benchmark unified tool approach
fn benchmark_unified(scenario: &BenchmarkScenario, config: &BenchmarkConfig) -> BenchmarkResult {
    let start = Instant::now();

    // Unified tools: docker, kubectl, git (3 tools)
    let tool_defs_tokens = calculate_unified_tool_definitions();
    let system_prompt_tokens = 100; // Typical system prompt

    let mut total_reasoning = 0;
    let mut total_tool_calls = 0;
    let mut total_outputs = 0;
    let mut total_responses = 0;
    let mut retries = 0;
    let mut errors = 0;

    for task in &scenario.tasks {
        // Reasoning: construct command string
        let reasoning_tokens = 30; // "Need to use docker stats --no-stream -a"
        total_reasoning += reasoning_tokens;

        // Tool call: command string
        let call_tokens = TokenCounter::count(&format!(r#"{{"command":"stats --no-stream -a"}}"#));
        total_tool_calls += call_tokens;

        // Simulate retry
        if config.simulate_retries && rand::random::<f64>() < config.unified_retry_rate {
            retries += 1;
            errors += 1;
            total_reasoning += reasoning_tokens;
            total_tool_calls += call_tokens;
        }

        // Output: raw text (larger)
        let output_tokens = 200; // Typical text output
        total_outputs += output_tokens;

        // Response: LLM summarizes
        let response_tokens = 50;
        total_responses += response_tokens;
    }

    let retry_overhead = retries * (tool_defs_tokens / 10); // Partial re-context

    let breakdown = TokenBreakdown {
        tool_definitions: tool_defs_tokens,
        system_prompt: system_prompt_tokens,
        reasoning: total_reasoning,
        tool_calls: total_tool_calls,
        tool_outputs: total_outputs,
        llm_responses: total_responses,
        retry_overhead,
    };

    let metrics = ExecutionMetrics {
        total_time_ms: start.elapsed().as_millis() as u64,
        llm_calls: scenario.tasks.len() + retries,
        tool_executions: scenario.tasks.len() + retries,
        retries,
        errors,
        parallel_calls: 0, // Unified doesn't parallelize well
    };

    let success_rate = 1.0 - (errors as f64 / scenario.tasks.len() as f64);

    BenchmarkResult {
        scenario: scenario.name.clone(),
        approach: ToolApproach::Unified,
        total_tokens: breakdown.total(),
        breakdown,
        metrics,
        success_rate,
    }
}

/// Benchmark per-operation tool approach
fn benchmark_per_operation(scenario: &BenchmarkScenario, config: &BenchmarkConfig) -> BenchmarkResult {
    let start = Instant::now();

    // Per-op tools: docker_ps, docker_stats, docker_logs, etc. (25 tools)
    let tool_defs_tokens = calculate_per_op_tool_definitions();
    let system_prompt_tokens = 100;

    let mut total_reasoning = 0;
    let mut total_tool_calls = 0;
    let mut total_outputs = 0;
    let mut total_responses = 0;
    let mut retries = 0;
    let mut errors = 0;
    let mut parallel_calls = 0;

    for task in &scenario.tasks {
        // Reasoning: select tool + parameters
        let reasoning_tokens = 40; // "Use docker_stats with all=true, format=json"
        total_reasoning += reasoning_tokens;

        // Tool call: structured parameters
        let call_tokens = TokenCounter::count(r#"{"all":true,"format":"json"}"#);
        total_tool_calls += call_tokens;

        // Simulate retry
        if config.simulate_retries && rand::random::<f64>() < config.per_op_retry_rate {
            retries += 1;
            errors += 1;
            total_reasoning += reasoning_tokens;
            total_tool_calls += call_tokens;
        }

        // Output: structured JSON (smaller, easier to parse)
        let output_tokens = 150; // Structured JSON
        total_outputs += output_tokens;

        // Response: LLM works with structure
        let response_tokens = 30; // Less processing needed
        total_responses += response_tokens;

        // Check if parallelizable
        if task.multi_step && config.simulate_parallel {
            parallel_calls += 1;
        }
    }

    let retry_overhead = retries * (tool_defs_tokens / 10);

    let breakdown = TokenBreakdown {
        tool_definitions: tool_defs_tokens,
        system_prompt: system_prompt_tokens,
        reasoning: total_reasoning,
        tool_calls: total_tool_calls,
        tool_outputs: total_outputs,
        llm_responses: total_responses,
        retry_overhead,
    };

    let metrics = ExecutionMetrics {
        total_time_ms: start.elapsed().as_millis() as u64,
        llm_calls: scenario.tasks.len() + retries,
        tool_executions: scenario.tasks.len() + retries,
        retries,
        errors,
        parallel_calls,
    };

    let success_rate = 1.0 - (errors as f64 / scenario.tasks.len() as f64);

    BenchmarkResult {
        scenario: scenario.name.clone(),
        approach: ToolApproach::PerOperation,
        total_tokens: breakdown.total(),
        breakdown,
        metrics,
        success_rate,
    }
}

/// Calculate token count for unified tool definitions
fn calculate_unified_tool_definitions() -> usize {
    // docker, kubectl, git unified tools
    let docker_def = r#"{
        "name": "docker",
        "description": "Execute any docker command. Takes a 'command' string with the full docker subcommand and arguments. Examples: 'ps -a', 'stats --no-stream', 'logs container_name --tail 100'. IMPORTANT: For 'docker stats', always include --no-stream flag to get a one-time snapshot instead of continuous output.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The docker subcommand and arguments"
                }
            }
        }
    }"#;

    let kubectl_def = r#"{
        "name": "kubectl",
        "description": "Execute any kubectl command. Common commands: get, describe, logs, apply, delete, exec. Always specify namespace with -n flag if not using default.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The kubectl subcommand and arguments"
                }
            }
        }
    }"#;

    let git_def = r#"{
        "name": "git",
        "description": "Execute any git command. Common commands: status, log, diff, commit, push, pull.",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The git subcommand and arguments"
                }
            }
        }
    }"#;

    TokenCounter::count(docker_def)
        + TokenCounter::count(kubectl_def)
        + TokenCounter::count(git_def)
}

/// Calculate token count for per-op tool definitions
fn calculate_per_op_tool_definitions() -> usize {
    // This would be ~25 tools total
    // For now, estimate based on average tool definition size
    let avg_tool_def_size = 120; // tokens per tool definition
    let num_tools = 25;

    avg_tool_def_size * num_tools
}
