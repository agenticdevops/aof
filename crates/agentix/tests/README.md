# AOFCTL Test Suite

Comprehensive test coverage for the kubectl-compatible CLI.

## Test Structure

```
tests/
├── unit/                   # Unit tests for individual components
│   ├── cli_parsing_tests.rs
│   ├── command_validation_tests.rs
│   ├── output_format_tests.rs
│   └── resource_type_tests.rs
├── integration/            # Integration tests for command execution
│   ├── run_command_tests.rs
│   ├── get_command_tests.rs
│   ├── apply_command_tests.rs
│   ├── delete_command_tests.rs
│   ├── validate_command_tests.rs
│   └── error_handling_tests.rs
├── acceptance/             # End-to-end acceptance tests
│   ├── kubectl_compatibility_tests.rs
│   └── end_to_end_tests.rs
├── fixtures/               # Test data and configurations
│   ├── simple_agent.yaml
│   ├── agent_with_tools.yaml
│   └── invalid_agent.yaml
├── test_helpers.rs         # Shared test utilities
└── README.md              # This file
```

## Test Categories

### Unit Tests
Test individual components in isolation:
- CLI command parsing
- Configuration validation
- Output formatting
- Resource type handling

### Integration Tests
Test command execution flow:
- Run command with different configurations
- Get command for various resources
- Apply command validation and storage
- Delete command with cleanup
- Validate command checks
- Error handling across commands

### Acceptance Tests
Test complete workflows and kubectl compatibility:
- Full agent lifecycle (validate → apply → run → get → delete)
- kubectl-style command patterns
- Multi-agent coordination
- End-to-end workflows

## Running Tests

### All Tests
```bash
cd aof/crates/aofctl
cargo test
```

### Unit Tests Only
```bash
cargo test --test unit
```

### Integration Tests Only
```bash
cargo test --test integration
```

### Acceptance Tests Only
```bash
cargo test --test acceptance
```

### Specific Test
```bash
cargo test test_run_command_with_valid_config
```

### With Output
```bash
cargo test -- --nocapture
```

### Test Coverage
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage
```

## Test Coverage Goals

- Statements: >80%
- Branches: >75%
- Functions: >80%
- Lines: >80%

## Test Fixtures

Located in `tests/fixtures/`:

- `simple_agent.yaml` - Basic agent configuration
- `agent_with_tools.yaml` - Agent with MCP tools
- `invalid_agent.yaml` - Invalid config for error testing

## Writing New Tests

### Unit Test Template
```rust
#[test]
fn test_feature_name() {
    // Arrange
    let input = setup_test_data();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected_value);
}
```

### Integration Test Template
```rust
#[tokio::test]
async fn test_command_integration() {
    // Setup
    let fixture = fixture_path("simple_agent.yaml");

    // Execute
    let result = execute_command(&fixture).await;

    // Verify
    assert!(result.is_ok());
}
```

### Acceptance Test Template
```rust
#[tokio::test]
async fn test_end_to_end_workflow() {
    // Create
    let config = create_test_config();

    // Execute workflow
    validate_command(&config).await?;
    apply_command(&config).await?;
    let result = run_command(&config, "test query").await?;

    // Verify
    assert!(result.contains("expected output"));

    // Cleanup
    delete_command("agent", "test-agent").await?;
}
```

## Test Helpers

Use `test_helpers.rs` for common test utilities:

```rust
use crate::test_helpers::*;

// Get fixture path
let config = fixture_path("simple_agent.yaml");

// Create temp directory
let temp = temp_dir();

// Use mock configs
let config_str = mock_agent_config();
```

## CI/CD Integration

Tests run automatically on:
- Pull requests
- Pushes to main/dev branches
- Release builds

## kubectl Compatibility Tests

These tests ensure our CLI matches kubectl patterns:

| aofctl Command | kubectl Equivalent |
|----------------|-------------------|
| `aofctl run --config agent.yaml --input "query"` | `kubectl run pod --image=nginx` |
| `aofctl get agents` | `kubectl get pods` |
| `aofctl get agent my-agent` | `kubectl get pod my-pod` |
| `aofctl apply --file config.yaml` | `kubectl apply -f deployment.yaml` |
| `aofctl delete agent my-agent` | `kubectl delete pod my-pod` |
| `aofctl describe agent my-agent` | `kubectl describe pod my-pod` |

## Troubleshooting

### Tests Failing
1. Ensure all dependencies are installed: `cargo build`
2. Check fixture files exist: `ls tests/fixtures/`
3. Verify test database is clean
4. Run with verbose output: `cargo test -- --nocapture`

### Coverage Issues
1. Ensure all error paths are tested
2. Add tests for edge cases
3. Test all output formats
4. Verify integration with runtime

## Future Test Enhancements

- [ ] Performance benchmarks
- [ ] Load testing
- [ ] Fuzz testing
- [ ] Property-based testing
- [ ] Mutation testing
- [ ] Security testing
