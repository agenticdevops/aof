/// Unit tests for output formatting
use serde_json;
use serde_yaml;

#[cfg(test)]
mod output_format_tests {
    use super::*;

    #[test]
    fn test_json_output_format() {
        let data = serde_json::json!({
            "success": true,
            "agent": "test-agent",
            "result": "test result"
        });

        let json_str = serde_json::to_string_pretty(&data).unwrap();
        assert!(json_str.contains("success"));
        assert!(json_str.contains("test-agent"));
    }

    #[test]
    fn test_yaml_output_format() {
        let data = serde_json::json!({
            "success": true,
            "agent": "test-agent",
            "result": "test result"
        });

        let yaml_str = serde_yaml::to_string(&data).unwrap();
        assert!(yaml_str.contains("success"));
        assert!(yaml_str.contains("test-agent"));
    }

    #[test]
    fn test_text_output_format() {
        let agent_name = "test-agent";
        let result = "test result";

        let text_output = format!("Agent: {}\nResult: {}", agent_name, result);
        assert!(text_output.contains("Agent: test-agent"));
        assert!(text_output.contains("Result: test result"));
    }

    #[test]
    fn test_json_parse_roundtrip() {
        let original = serde_json::json!({
            "success": true,
            "data": {
                "nested": "value"
            }
        });

        let json_str = serde_json::to_string(&original).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_yaml_parse_roundtrip() {
        let original = serde_json::json!({
            "success": true,
            "data": {
                "nested": "value"
            }
        });

        let yaml_str = serde_yaml::to_string(&original).unwrap();
        let parsed: serde_json::Value = serde_yaml::from_str(&yaml_str).unwrap();

        assert_eq!(original, parsed);
    }
}
