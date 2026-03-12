/// TDD test suite for AgentSpec and WorkspaceConfig deserialization and validation.
///
/// Covers the OpenAgentiX v1 agent YAML spec (openagentix.dev/v1, kind: Agent)
/// and workspace config (openagentix.dev/v1, kind: Workspace).
use agentix_core::{
    AgentMode, AgentSpec, GatewayConfig, McpTransportType, NotificationChannel, ProviderConfig,
    SpecToolType, WorkspaceConfig,
};
use std::collections::HashMap;

// ============================================================================
// Deserialization Tests
// ============================================================================

#[test]
fn test_minimal_agent_yaml() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: hello-world
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: |
    You are a helpful assistant.
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse minimal yaml");

    assert_eq!(spec.api_version, "openagentix.dev/v1");
    assert_eq!(spec.kind, "Agent");
    assert_eq!(spec.metadata.name, "hello-world");
    assert_eq!(
        spec.spec.model.as_deref(),
        Some("anthropic/claude-sonnet-4-6")
    );
    assert!(spec.spec.system_prompt.is_some());
    assert_eq!(spec.spec.max_iterations, 10);
    assert_eq!(spec.spec.timeout, "5m");
    assert_eq!(spec.spec.mode, AgentMode::Autonomous);
}

#[test]
fn test_full_agent_yaml() {
    let yaml = r##"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: db-optimizer
  namespace: production
  version: "1.2.0"
  labels:
    team: platform
    pack: dba-optimizer
  annotations:
    description: "Analyzes slow PostgreSQL queries"
    runbook: "https://wiki.example.com/agents/db-optimizer"
spec:
  model: anthropic/claude-sonnet-4-6
  mode: semi-autonomous
  system_prompt: |
    You are a PostgreSQL performance expert.
  max_iterations: 20
  timeout: "15m"
  tools:
    - name: run-explain
      type: cli
      description: "Run EXPLAIN ANALYZE"
      command: psql
      args:
        - -U
        - postgres
  mcp_servers:
    - name: postgres-mcp
      transport: stdio
      command: npx
      args:
        - "-y"
        - "@modelcontextprotocol/server-postgres"
      env:
        DATABASE_URL: "${DATABASE_URL}"
  notifications:
    - channel: slack
      target: "#db-alerts"
  approval:
    required_for:
      - "psql * CREATE INDEX *"
    approvers:
      - "@db-admin"
    timeout: "1h"
  budget:
    daily_limit: "$2.00"
    max_tokens_per_run: 100000
  telemetry:
    enabled: true
    exporter: otlp
  env:
    DB_USER: "${DB_USER}"
    LOG_LEVEL: info
"##;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse full yaml");

    assert_eq!(spec.metadata.name, "db-optimizer");
    assert_eq!(spec.metadata.namespace.as_deref(), Some("production"));
    assert_eq!(spec.metadata.version.as_deref(), Some("1.2.0"));
    assert_eq!(spec.metadata.labels.get("team").map(|s| s.as_str()), Some("platform"));
    assert_eq!(spec.spec.mode, AgentMode::SemiAutonomous);
    assert_eq!(spec.spec.max_iterations, 20);
    assert_eq!(spec.spec.timeout, "15m");
    assert_eq!(spec.spec.tools.len(), 1);
    assert_eq!(spec.spec.mcp_servers.len(), 1);
    assert_eq!(spec.spec.notifications.len(), 1);
    assert!(spec.spec.approval.is_some());
    assert!(spec.spec.budget.is_some());
    assert!(spec.spec.telemetry.is_some());
    assert_eq!(spec.spec.env.get("LOG_LEVEL").map(|s| s.as_str()), Some("info"));
}

#[test]
fn test_unified_tools_list() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: multi-tool-agent
spec:
  model: openai/gpt-4o
  system_prompt: "Test agent"
  tools:
    - name: kubectl-get-pods
      type: cli
      description: "List all pods"
      command: kubectl
      args:
        - get
        - pods
    - name: db-schema
      type: mcp
      description: "Inspect database schema"
      server: postgres-mcp
    - name: check-disk-usage
      type: shell
      description: "Check disk usage"
      command: "df -h {{path}}"
  mcp_servers:
    - name: postgres-mcp
      transport: stdio
      command: npx
      args: ["-y", "@modelcontextprotocol/server-postgres"]
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse tools yaml");

    assert_eq!(spec.spec.tools.len(), 3);

    let cli_tool = &spec.spec.tools[0];
    assert_eq!(cli_tool.tool_type, SpecToolType::Cli);
    assert_eq!(cli_tool.command.as_deref(), Some("kubectl"));
    assert_eq!(cli_tool.args, vec!["get", "pods"]);

    let mcp_tool = &spec.spec.tools[1];
    assert_eq!(mcp_tool.tool_type, SpecToolType::Mcp);
    assert_eq!(mcp_tool.server.as_deref(), Some("postgres-mcp"));

    let shell_tool = &spec.spec.tools[2];
    assert_eq!(shell_tool.tool_type, SpecToolType::Shell);
    assert_eq!(shell_tool.command.as_deref(), Some("df -h {{path}}"));
}

#[test]
fn test_system_prompt_file() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: security-scanner
spec:
  model: openai/gpt-4o
  system_prompt_file: prompts/security-scanner.md
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse system_prompt_file yaml");

    assert!(spec.spec.system_prompt.is_none());
    assert_eq!(
        spec.spec.system_prompt_file.as_deref(),
        Some("prompts/security-scanner.md")
    );
}

#[test]
fn test_metadata_namespace_labels() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: scanner
  namespace: security
  version: "2.0.1"
  labels:
    team: security
    severity: critical
  annotations:
    owner: "security-team@example.com"
spec:
  model: openai/gpt-4o
  system_prompt: "Security scanner"
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse metadata yaml");

    assert_eq!(spec.metadata.namespace.as_deref(), Some("security"));
    assert_eq!(spec.metadata.version.as_deref(), Some("2.0.1"));
    assert_eq!(spec.metadata.labels.len(), 2);
    assert_eq!(spec.metadata.labels.get("severity").map(|s| s.as_str()), Some("critical"));
    assert_eq!(spec.metadata.annotations.len(), 1);
}

#[test]
fn test_provider_model_format() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse model yaml");

    let (provider, model_name) = spec.parse_model().expect("should extract provider/model");
    assert_eq!(provider, "anthropic");
    assert_eq!(model_name, "claude-sonnet-4-6");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_invalid_name_uppercase() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: MyAgent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for uppercase name");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("metadata.name"), "error should mention field: {}", err);
}

#[test]
fn test_invalid_name_too_long() {
    let long_name = "a".repeat(64);
    let yaml = format!(
        r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: {}
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
"#,
        long_name
    );

    let spec = AgentSpec::from_yaml(&yaml).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for name > 63 chars");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("metadata.name"), "error should mention field: {}", err);
}

#[test]
fn test_both_system_prompts_fails() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Inline prompt"
  system_prompt_file: prompts/test.md
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for both system prompts");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("system_prompt"),
        "error should mention system_prompt: {}",
        err
    );
}

#[test]
fn test_invalid_model_format() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: claude-sonnet-4-6
  system_prompt: "Test"
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for model without /");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("spec.model"), "error should mention spec.model: {}", err);
}

#[test]
fn test_max_iterations_out_of_range() {
    // max_iterations = 0 (below minimum 1)
    let yaml_zero = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
  max_iterations: 0
"#;

    let spec = AgentSpec::from_yaml(yaml_zero).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for max_iterations=0");

    // max_iterations = 101 (above maximum 100)
    let yaml_over = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
  max_iterations: 101
"#;

    let spec = AgentSpec::from_yaml(yaml_over).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for max_iterations=101");
}

#[test]
fn test_wrong_api_version() {
    let yaml = r#"
apiVersion: kubernetes.io/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for wrong apiVersion");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("apiVersion"), "error should mention apiVersion: {}", err);
}

#[test]
fn test_wrong_kind() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Pod
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
"#;

    let spec = AgentSpec::from_yaml(yaml).expect("yaml should parse");
    let result = spec.validate();
    assert!(result.is_err(), "validation should fail for wrong kind");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("kind"), "error should mention kind: {}", err);
}

// ============================================================================
// Workspace Config Tests
// ============================================================================

#[test]
fn test_workspace_config_deserialize() {
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: platform-ops
  labels:
    environment: production
  annotations:
    owner: "platform@example.com"
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 15
    timeout: "10m"
    mode: autonomous
  providers:
    anthropic:
      api_key: "${ANTHROPIC_API_KEY}"
    openai:
      api_key: "${OPENAI_API_KEY}"
      base_url: "https://api.openai.com/v1"
    ollama:
      base_url: "http://localhost:11434"
  gateway:
    host: "0.0.0.0"
    port: 7777
  agents_dir: "./agents"
"#;

    let ws = WorkspaceConfig::from_yaml(yaml).expect("should parse workspace yaml");

    assert_eq!(ws.api_version, "openagentix.dev/v1");
    assert_eq!(ws.kind, "Workspace");
    assert_eq!(ws.metadata.name, "platform-ops");
    assert_eq!(ws.metadata.labels.get("environment").map(|s| s.as_str()), Some("production"));
    assert_eq!(
        ws.spec.defaults.model.as_deref(),
        Some("anthropic/claude-sonnet-4-6")
    );
    assert_eq!(ws.spec.defaults.max_iterations, Some(15));
    assert_eq!(ws.spec.defaults.timeout.as_deref(), Some("10m"));
    assert_eq!(ws.spec.defaults.mode, Some(AgentMode::Autonomous));
    assert!(ws.spec.providers.contains_key("anthropic"));
    assert!(ws.spec.providers.contains_key("openai"));
    assert_eq!(ws.spec.gateway.host, "0.0.0.0");
    assert_eq!(ws.spec.gateway.port, 7777);
    assert_eq!(ws.spec.agents_dir, "./agents");
}

#[test]
fn test_workspace_defaults_merge() {
    let workspace_yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test-workspace
spec:
  defaults:
    model: anthropic/claude-sonnet-4-6
    max_iterations: 20
    timeout: "30m"
"#;

    // Agent without model — should inherit from workspace
    let agent_no_model_yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: no-model-agent
spec:
  system_prompt: "Test"
"#;

    // Agent with explicit model — should override workspace
    let agent_with_model_yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: has-model-agent
spec:
  model: openai/gpt-4o
  system_prompt: "Test"
"#;

    let ws = WorkspaceConfig::from_yaml(workspace_yaml).expect("should parse workspace");

    let mut agent_no_model =
        AgentSpec::from_yaml(agent_no_model_yaml).expect("should parse agent");
    agent_no_model.merge_workspace_defaults(&ws);
    assert_eq!(
        agent_no_model.spec.model.as_deref(),
        Some("anthropic/claude-sonnet-4-6"),
        "should inherit model from workspace"
    );
    assert_eq!(
        agent_no_model.spec.max_iterations, 20,
        "should inherit max_iterations from workspace"
    );
    assert_eq!(
        agent_no_model.spec.timeout, "30m",
        "should inherit timeout from workspace"
    );

    let mut agent_with_model =
        AgentSpec::from_yaml(agent_with_model_yaml).expect("should parse agent");
    agent_with_model.merge_workspace_defaults(&ws);
    assert_eq!(
        agent_with_model.spec.model.as_deref(),
        Some("openai/gpt-4o"),
        "agent model should not be overridden by workspace"
    );
}

#[test]
fn test_env_var_expansion() {
    // Set a test env var
    std::env::set_var("TEST_ANTHROPIC_KEY_ABC123", "sk-ant-test-key-value");

    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Workspace
metadata:
  name: test-workspace
spec:
  providers:
    anthropic:
      api_key: "${TEST_ANTHROPIC_KEY_ABC123}"
"#;

    let mut ws = WorkspaceConfig::from_yaml(yaml).expect("should parse workspace");
    ws.expand_env_vars();

    let anthropic = ws.spec.providers.get("anthropic").expect("should have anthropic provider");
    assert_eq!(
        anthropic.api_key.as_deref(),
        Some("sk-ant-test-key-value"),
        "env var should be expanded"
    );

    // Clean up
    std::env::remove_var("TEST_ANTHROPIC_KEY_ABC123");
}

// ============================================================================
// Placeholder Fields Round-Trip Test
// ============================================================================

#[test]
fn test_placeholder_fields_round_trip() {
    let yaml = r##"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
  triggers:
    - type: cron
      schedule: "0 2 * * *"
    - type: manual
  approval:
    required_for:
      - "kubectl delete *"
    approvers:
      - "@ops-lead"
    timeout: "30m"
  budget:
    daily_limit: "$0.50"
    max_tokens_per_run: 50000
  telemetry:
    enabled: true
    exporter: otlp
"##;

    let spec = AgentSpec::from_yaml(yaml).expect("should parse placeholder yaml");

    // Verify placeholder fields are preserved
    assert_eq!(spec.spec.triggers.len(), 2);
    assert!(spec.spec.approval.is_some());
    assert!(spec.spec.budget.is_some());
    assert!(spec.spec.telemetry.is_some());

    // Round-trip: serialize and re-parse
    let serialized = serde_yaml::to_string(&spec).expect("should serialize");
    let reparsed = AgentSpec::from_yaml(&serialized).expect("should re-parse after serialization");

    assert_eq!(reparsed.spec.triggers.len(), 2);
    assert!(reparsed.spec.approval.is_some());
    assert!(reparsed.spec.budget.is_some());
    assert!(reparsed.spec.telemetry.is_some());
}

// ============================================================================
// Error Message Tests
// ============================================================================

#[test]
fn test_parse_error_shows_field_path() {
    // max_iterations expects a u32, give it a string to trigger a parse error
    let yaml = r#"
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: test-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "Test"
  max_iterations: "not-a-number"
"#;

    let result = AgentSpec::from_yaml(yaml);
    assert!(result.is_err(), "should fail to parse invalid max_iterations type");
    let err = result.unwrap_err().to_string();
    // The error should contain the field path
    assert!(
        err.contains("max_iterations") || err.contains("spec"),
        "error should contain field path context: {}",
        err
    );
}
