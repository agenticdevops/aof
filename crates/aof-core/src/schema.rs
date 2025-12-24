use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AofError, AofResult};

/// Output schema for structured LLM responses
///
/// Similar to Agno's output_schema, this defines the expected structure
/// of agent responses. The LLM will be instructed to produce output
/// matching this schema.
///
/// # Examples
///
/// ```rust
/// use aof_core::schema::OutputSchema;
/// use serde_json::json;
///
/// // Define a schema for container information
/// let schema = OutputSchema::from_json_schema(json!({
///     "type": "object",
///     "properties": {
///         "containers": {
///             "type": "array",
///             "items": {
///                 "type": "object",
///                 "properties": {
///                     "name": {"type": "string"},
///                     "status": {"type": "string"},
///                     "uptime": {"type": "string"}
///                 },
///                 "required": ["name", "status"]
///             }
///         }
///     },
///     "required": ["containers"]
/// }));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSchema {
    /// JSON Schema definition
    pub schema: Value,

    /// Optional description of what this schema represents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether to use strict mode (enforce exact schema match)
    #[serde(default = "default_strict")]
    pub strict: bool,

    /// Format hint for rendering (table, list, json, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_hint: Option<FormatHint>,
}

fn default_strict() -> bool {
    true
}

/// Hint for how to render structured output
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FormatHint {
    /// Render as a table (for arrays of objects)
    Table,
    /// Render as a bulleted list
    List,
    /// Render as JSON (pretty-printed)
    Json,
    /// Render as YAML
    Yaml,
    /// Auto-detect best format
    Auto,
}

impl OutputSchema {
    /// Create from a JSON Schema definition
    pub fn from_json_schema(schema: Value) -> Self {
        Self {
            schema,
            description: None,
            strict: true,
            format_hint: Some(FormatHint::Auto),
        }
    }

    /// Create with description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Set format hint
    pub fn with_format_hint(mut self, hint: FormatHint) -> Self {
        self.format_hint = Some(hint);
        self
    }

    /// Validate output against this schema
    pub fn validate(&self, output: &Value) -> AofResult<()> {
        // For now, just check if it's valid JSON
        // TODO: Implement full JSON Schema validation using jsonschema crate
        if !self.strict {
            return Ok(());
        }

        // Basic validation: check that output matches schema type
        let schema_type = self.schema.get("type")
            .and_then(|t| t.as_str());

        let output_type = match output {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };

        if let Some(expected) = schema_type {
            if expected != output_type {
                return Err(AofError::validation(format!(
                    "Schema validation failed: expected type '{}', got '{}'",
                    expected, output_type
                )));
            }
        }

        Ok(())
    }

    /// Convert to LLM tool definition for structured output
    ///
    /// Uses the "function calling" approach where the LLM calls a pseudo-function
    /// with structured arguments matching the schema.
    pub fn to_llm_tool(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": "respond_with_structured_output",
                "description": self.description.as_deref()
                    .unwrap_or("Respond with structured output matching the provided schema"),
                "parameters": self.schema.clone()
            }
        })
    }

    /// Get system prompt instructions for structured output
    pub fn to_system_instructions(&self) -> String {
        let mut instructions = String::from(
            "You MUST format your response as structured JSON matching this schema:\n\n"
        );

        if let Some(desc) = &self.description {
            instructions.push_str(&format!("Description: {}\n\n", desc));
        }

        instructions.push_str(&format!(
            "Schema:\n{}\n\n",
            serde_json::to_string_pretty(&self.schema).unwrap_or_default()
        ));

        instructions.push_str(
            "Respond ONLY with valid JSON matching this schema. Do not include any explanation or markdown formatting."
        );

        instructions
    }
}

/// Input schema for validating agent inputs
///
/// Similar to Agno's structured input, this validates input data
/// before passing it to the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSchema {
    /// JSON Schema definition
    pub schema: Value,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl InputSchema {
    /// Create from JSON Schema
    pub fn from_json_schema(schema: Value) -> Self {
        Self {
            schema,
            description: None,
        }
    }

    /// Validate input against schema
    pub fn validate(&self, _input: &Value) -> AofResult<()> {
        // TODO: Implement full JSON Schema validation
        Ok(())
    }
}

/// Common output schemas for typical use cases
pub mod schemas {
    use super::*;
    use serde_json::json;

    /// Schema for container list (docker ps, kubectl get pods)
    pub fn container_list() -> OutputSchema {
        OutputSchema::from_json_schema(json!({
            "type": "object",
            "properties": {
                "containers": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Container name"
                            },
                            "image": {
                                "type": "string",
                                "description": "Container image"
                            },
                            "status": {
                                "type": "string",
                                "description": "Container status (Running, Exited, etc.)"
                            },
                            "uptime": {
                                "type": "string",
                                "description": "How long the container has been running"
                            }
                        },
                        "required": ["name", "status"]
                    }
                }
            },
            "required": ["containers"]
        }))
        .with_description("List of running containers with status information")
        .with_format_hint(FormatHint::Table)
    }

    /// Schema for resource stats (docker stats, kubectl top)
    pub fn resource_stats() -> OutputSchema {
        OutputSchema::from_json_schema(json!({
            "type": "object",
            "properties": {
                "resources": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string"
                            },
                            "cpu_percent": {
                                "type": "number",
                                "description": "CPU usage percentage"
                            },
                            "memory_usage": {
                                "type": "string",
                                "description": "Memory usage (e.g., '128MiB')"
                            },
                            "memory_limit": {
                                "type": "string",
                                "description": "Memory limit (e.g., '1GiB')"
                            },
                            "memory_percent": {
                                "type": "number",
                                "description": "Memory usage percentage"
                            }
                        },
                        "required": ["name", "cpu_percent", "memory_usage"]
                    }
                }
            },
            "required": ["resources"]
        }))
        .with_description("Resource usage statistics")
        .with_format_hint(FormatHint::Table)
    }

    /// Schema for simple list output
    pub fn simple_list() -> OutputSchema {
        OutputSchema::from_json_schema(json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                }
            },
            "required": ["items"]
        }))
        .with_description("Simple list of items")
        .with_format_hint(FormatHint::List)
    }

    /// Schema for key-value output
    pub fn key_value() -> OutputSchema {
        OutputSchema::from_json_schema(json!({
            "type": "object",
            "properties": {
                "data": {
                    "type": "object",
                    "additionalProperties": true
                }
            },
            "required": ["data"]
        }))
        .with_description("Key-value data")
        .with_format_hint(FormatHint::Json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_output_schema_from_json() {
        let schema = OutputSchema::from_json_schema(json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        }));

        assert!(schema.strict);
        assert_eq!(schema.format_hint, Some(FormatHint::Auto));
    }

    #[test]
    fn test_output_schema_validation_success() {
        let schema = OutputSchema::from_json_schema(json!({
            "type": "object"
        }));

        let output = json!({"name": "test"});
        assert!(schema.validate(&output).is_ok());
    }

    #[test]
    fn test_output_schema_validation_failure() {
        let schema = OutputSchema::from_json_schema(json!({
            "type": "object"
        }));

        let output = json!("not an object");
        assert!(schema.validate(&output).is_err());
    }

    #[test]
    fn test_output_schema_to_llm_tool() {
        let schema = OutputSchema::from_json_schema(json!({
            "type": "object",
            "properties": {
                "result": {"type": "string"}
            }
        }))
        .with_description("Test schema");

        let tool = schema.to_llm_tool();
        assert_eq!(tool["type"], "function");
        assert_eq!(tool["function"]["name"], "respond_with_structured_output");
    }

    #[test]
    fn test_container_list_schema() {
        let schema = schemas::container_list();
        assert_eq!(schema.format_hint, Some(FormatHint::Table));
        assert!(schema.description.is_some());
    }

    #[test]
    fn test_resource_stats_schema() {
        let schema = schemas::resource_stats();
        assert_eq!(schema.format_hint, Some(FormatHint::Table));

        // Validate a sample output
        let sample = json!({
            "resources": [
                {
                    "name": "container1",
                    "cpu_percent": 25.5,
                    "memory_usage": "128MiB",
                    "memory_percent": 10.2
                }
            ]
        });

        assert!(schema.validate(&sample).is_ok());
    }
}
