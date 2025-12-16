/// Common test utilities and helpers
use std::path::{Path, PathBuf};

/// Get path to test fixtures directory
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Get path to a specific fixture file
pub fn fixture_path(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

/// Mock agent config for testing
pub fn mock_agent_config() -> String {
    r#"
name: test-agent
model: claude-3-5-sonnet-20241022
provider: anthropic
max_iterations: 5
temperature: 0.7
system_prompt: "You are a test agent"
tools: []
"#
    .to_string()
}

/// Mock agent config with tools
pub fn mock_agent_config_with_tools() -> String {
    r#"
name: agent-with-tools
model: claude-3-5-sonnet-20241022
provider: anthropic
max_iterations: 10
temperature: 0.5
max_tokens: 4096
system_prompt: "You are an agent with tool access"
tools:
  - calculator
  - file_reader
memory: redis://localhost:6379
"#
    .to_string()
}

/// Mock invalid agent config
pub fn mock_invalid_agent_config() -> String {
    r#"
name: ""
model: ""
max_iterations: 0
temperature: 3.0
"#
    .to_string()
}
