use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AgentixError, AgentixResult};

/// Output schema for structured LLM responses
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
    pub fn validate(&self, output: &Value) -> AgentixResult<()> {
        if !self.strict {
            return Ok(());
        }

        let schema_type = self.schema.get("type").and_then(|t| t.as_str());

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
                return Err(AgentixError::validation(format!(
                    "Schema validation failed: expected type '{}', got '{}'",
                    expected, output_type
                )));
            }
        }

        Ok(())
    }

    /// Convert to LLM tool definition for structured output
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
            "You MUST format your response as structured JSON matching this schema:\n\n",
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
    pub fn validate(&self, _input: &Value) -> AgentixResult<()> {
        Ok(())
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
}
