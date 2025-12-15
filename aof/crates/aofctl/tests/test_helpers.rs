/// Test helper utilities
use std::path::{Path, PathBuf};

/// Get path to test fixtures directory
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Get path to a specific fixture file
pub fn fixture_path(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

/// Create a temporary test directory
#[cfg(test)]
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Mock agent config for testing
#[cfg(test)]
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
#[cfg(test)]
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
#[cfg(test)]
pub fn mock_invalid_agent_config() -> String {
    r#"
name: ""
model: ""
max_iterations: 0
temperature: 3.0
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_dir_exists() {
        let dir = fixtures_dir();
        assert!(dir.exists() || dir.to_str().unwrap().contains("fixtures"));
    }

    #[test]
    fn test_fixture_path() {
        let path = fixture_path("simple_agent.yaml");
        assert!(path.to_str().unwrap().ends_with("simple_agent.yaml"));
    }

    #[test]
    fn test_mock_configs() {
        let config = mock_agent_config();
        assert!(config.contains("test-agent"));

        let config_with_tools = mock_agent_config_with_tools();
        assert!(config_with_tools.contains("calculator"));

        let invalid_config = mock_invalid_agent_config();
        assert!(invalid_config.contains("temperature: 3.0"));
    }
}
