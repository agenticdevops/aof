// AOF Core - Standalone Trigger resource type
//
// Trigger represents a decoupled message source that can be shared across
// multiple flows via FlowBindings. This enables:
// - Reusing the same trigger configuration across flows
// - Separating trigger concerns from flow logic
// - Multi-tenant deployments with different routing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trigger - Standalone message source
///
/// Example:
/// ```yaml
/// apiVersion: aof.dev/v1
/// kind: Trigger
/// metadata:
///   name: slack-prod-channel
/// spec:
///   type: Slack
///   config:
///     bot_token: ${SLACK_BOT_TOKEN}
///     signing_secret: ${SLACK_SIGNING_SECRET}
///     channels: [production, prod-alerts]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    /// API version (e.g., "aof.dev/v1")
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Resource kind, always "Trigger"
    #[serde(default = "default_trigger_kind")]
    pub kind: String,

    /// Trigger metadata
    pub metadata: TriggerMetadata,

    /// Trigger specification
    pub spec: TriggerSpec,
}

fn default_api_version() -> String {
    "aof.dev/v1".to_string()
}

fn default_trigger_kind() -> String {
    "Trigger".to_string()
}

/// Trigger metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMetadata {
    /// Trigger name (unique identifier)
    pub name: String,

    /// Namespace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Labels for categorization
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// Annotations for additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub annotations: HashMap<String, String>,
}

/// Trigger specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerSpec {
    /// Trigger type (Slack, Telegram, Discord, HTTP, Schedule, etc.)
    #[serde(rename = "type")]
    pub trigger_type: StandaloneTriggerType,

    /// Trigger-specific configuration
    #[serde(default)]
    pub config: StandaloneTriggerConfig,

    /// Whether this trigger is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Command bindings for this trigger
    /// Maps slash commands to agents, fleets, or flows
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub commands: HashMap<String, CommandBinding>,

    /// Default agent for @mentions and natural language messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
}

/// Command binding - routes a slash command to an agent, fleet, or flow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandBinding {
    /// Route to a specific agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Route to a fleet (multi-agent team)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fleet: Option<String>,

    /// Route to a flow (multi-step workflow)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,

    /// Description for help text
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

fn default_enabled() -> bool {
    true
}

/// Types of standalone triggers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StandaloneTriggerType {
    /// Slack events (mentions, messages, slash commands)
    Slack,
    /// Telegram bot events
    Telegram,
    /// Discord bot events
    Discord,
    /// WhatsApp Business API events
    WhatsApp,
    /// Generic HTTP webhook
    HTTP,
    /// Cron/schedule-based trigger
    Schedule,
    /// PagerDuty incidents
    PagerDuty,
    /// GitHub webhooks
    GitHub,
    /// Jira webhooks
    Jira,
    /// Manual trigger (CLI invocation)
    Manual,
}

impl std::fmt::Display for StandaloneTriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slack => write!(f, "slack"),
            Self::Telegram => write!(f, "telegram"),
            Self::Discord => write!(f, "discord"),
            Self::WhatsApp => write!(f, "whatsapp"),
            Self::HTTP => write!(f, "http"),
            Self::Schedule => write!(f, "schedule"),
            Self::PagerDuty => write!(f, "pagerduty"),
            Self::GitHub => write!(f, "github"),
            Self::Jira => write!(f, "jira"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

/// Trigger-specific configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StandaloneTriggerConfig {
    // ============================================
    // Chat Platform Common Fields
    // ============================================

    /// Bot token (or env var reference ${VAR_NAME})
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token: Option<String>,

    /// Signing secret (Slack)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_secret: Option<String>,

    /// App secret (Discord)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_secret: Option<String>,

    /// Events to listen for
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<String>,

    /// Channels to listen on (names or IDs)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<String>,

    /// Chat IDs (Telegram)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chat_ids: Vec<i64>,

    /// Guild IDs (Discord)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guild_ids: Vec<String>,

    /// Users to respond to (user IDs)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<String>,

    /// Message patterns to match (regex)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<String>,

    // ============================================
    // HTTP Webhook Fields
    // ============================================

    /// HTTP path pattern (for HTTP trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// HTTP methods to accept
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<String>,

    /// Required headers for authentication
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub required_headers: HashMap<String, String>,

    /// Webhook secret for signature verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_secret: Option<String>,

    // ============================================
    // Schedule Fields
    // ============================================

    /// Cron expression (for Schedule trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,

    /// Timezone (for Schedule trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    // ============================================
    // PagerDuty Fields
    // ============================================

    /// PagerDuty API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// PagerDuty routing key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_key: Option<String>,

    /// PagerDuty service IDs to monitor
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub service_ids: Vec<String>,

    // ============================================
    // GitHub Fields
    // ============================================

    /// GitHub webhook events (push, pull_request, issues, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub github_events: Vec<String>,

    /// Repository filter (owner/repo)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repositories: Vec<String>,

    // ============================================
    // WhatsApp Fields
    // ============================================

    /// WhatsApp Business Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_account_id: Option<String>,

    /// WhatsApp phone number ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number_id: Option<String>,

    /// WhatsApp verify token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_token: Option<String>,

    // ============================================
    // Common Fields
    // ============================================

    /// Port to listen on (for webhook-based triggers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Host to bind to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Additional configuration
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Trigger {
    /// Get the trigger name
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the trigger type
    pub fn trigger_type(&self) -> StandaloneTriggerType {
        self.spec.trigger_type
    }

    /// Validate the trigger configuration
    /// Note: This validates structure, not that env vars are set (they're expanded at runtime)
    pub fn validate(&self) -> Result<(), String> {
        // Check name
        if self.metadata.name.is_empty() {
            return Err("Trigger name is required".to_string());
        }

        // Type-specific validation - check that required fields are present
        // (env var references like ${VAR} are valid - they'll be expanded later)
        match self.spec.trigger_type {
            StandaloneTriggerType::Slack => {
                if self.spec.config.bot_token.is_none() {
                    return Err("Slack trigger requires bot_token".to_string());
                }
            }
            StandaloneTriggerType::Telegram => {
                if self.spec.config.bot_token.is_none() {
                    return Err("Telegram trigger requires bot_token".to_string());
                }
            }
            StandaloneTriggerType::Discord => {
                if self.spec.config.bot_token.is_none() {
                    return Err("Discord trigger requires bot_token".to_string());
                }
            }
            StandaloneTriggerType::Schedule => {
                if self.spec.config.cron.is_none() {
                    return Err("Schedule trigger requires cron expression".to_string());
                }
            }
            StandaloneTriggerType::PagerDuty => {
                if self.spec.config.api_key.is_none() && self.spec.config.routing_key.is_none() {
                    return Err("PagerDuty trigger requires api_key or routing_key".to_string());
                }
            }
            StandaloneTriggerType::WhatsApp => {
                if self.spec.config.bot_token.is_none() {
                    return Err("WhatsApp trigger requires bot_token (access token)".to_string());
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Expand environment variables in configuration
    pub fn expand_env_vars(&mut self) {
        let config = &mut self.spec.config;

        if let Some(ref token) = config.bot_token {
            config.bot_token = Some(expand_env_var(token));
        }
        if let Some(ref secret) = config.signing_secret {
            config.signing_secret = Some(expand_env_var(secret));
        }
        if let Some(ref secret) = config.app_secret {
            config.app_secret = Some(expand_env_var(secret));
        }
        if let Some(ref secret) = config.webhook_secret {
            config.webhook_secret = Some(expand_env_var(secret));
        }
        if let Some(ref key) = config.api_key {
            config.api_key = Some(expand_env_var(key));
        }
        if let Some(ref key) = config.routing_key {
            config.routing_key = Some(expand_env_var(key));
        }
        if let Some(ref token) = config.verify_token {
            config.verify_token = Some(expand_env_var(token));
        }
    }

    /// Check if a message matches this trigger's filters
    pub fn matches(&self, platform: &str, channel: Option<&str>, user: Option<&str>, text: Option<&str>) -> bool {
        // Platform must match
        let trigger_platform = self.spec.trigger_type.to_string().to_lowercase();
        if trigger_platform != platform.to_lowercase() {
            return false;
        }

        let config = &self.spec.config;

        // Channel filter (if specified)
        if !config.channels.is_empty() {
            if let Some(ch) = channel {
                if !config.channels.iter().any(|c| c == ch) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // User filter (if specified)
        if !config.users.is_empty() {
            if let Some(u) = user {
                if !config.users.iter().any(|allowed| allowed == u) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Pattern filter (if specified)
        if !config.patterns.is_empty() {
            if let Some(t) = text {
                let matches_pattern = config.patterns.iter().any(|p| {
                    if let Ok(re) = regex::Regex::new(p) {
                        re.is_match(t)
                    } else {
                        t.contains(p)
                    }
                });
                if !matches_pattern {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Calculate a match score for routing priority
    /// Higher score = more specific match = higher priority
    pub fn match_score(&self, platform: &str, channel: Option<&str>, user: Option<&str>, text: Option<&str>) -> u32 {
        if !self.matches(platform, channel, user, text) {
            return 0;
        }

        let config = &self.spec.config;
        let mut score = 10; // Base score for platform match

        // Channel specificity
        if !config.channels.is_empty() && channel.is_some() {
            score += 100;
        }

        // User specificity
        if !config.users.is_empty() && user.is_some() {
            score += 80;
        }

        // Pattern specificity
        if !config.patterns.is_empty() && text.is_some() {
            score += 60;
        }

        score
    }
}

/// Expand ${VAR_NAME} patterns in a string
fn expand_env_var(value: &str) -> String {
    let mut result = value.to_string();
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();

    for cap in re.captures_iter(value) {
        let var_name = &cap[1];
        if let Ok(var_value) = std::env::var(var_name) {
            result = result.replace(&cap[0], &var_value);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slack_trigger() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: slack-prod-channel
  labels:
    environment: production
spec:
  type: Slack
  config:
    bot_token: ${SLACK_BOT_TOKEN}
    signing_secret: ${SLACK_SIGNING_SECRET}
    channels:
      - production
      - prod-alerts
    events:
      - app_mention
      - message
"#;

        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(trigger.metadata.name, "slack-prod-channel");
        assert_eq!(trigger.spec.trigger_type, StandaloneTriggerType::Slack);
        assert_eq!(trigger.spec.config.channels.len(), 2);
        assert!(trigger.validate().is_ok());
    }

    #[test]
    fn test_parse_telegram_trigger() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: telegram-oncall
spec:
  type: Telegram
  config:
    bot_token: ${TELEGRAM_BOT_TOKEN}
    chat_ids:
      - -1001234567890
    users:
      - "123456789"
"#;

        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(trigger.spec.trigger_type, StandaloneTriggerType::Telegram);
        assert_eq!(trigger.spec.config.chat_ids.len(), 1);
        assert!(trigger.validate().is_ok());
    }

    #[test]
    fn test_parse_schedule_trigger() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: daily-report
spec:
  type: Schedule
  config:
    cron: "0 9 * * *"
    timezone: "America/New_York"
"#;

        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(trigger.spec.trigger_type, StandaloneTriggerType::Schedule);
        assert_eq!(trigger.spec.config.cron, Some("0 9 * * *".to_string()));
        assert!(trigger.validate().is_ok());
    }

    #[test]
    fn test_parse_http_trigger() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: webhook-endpoint
spec:
  type: HTTP
  config:
    path: /webhook/github
    methods:
      - POST
    webhook_secret: ${WEBHOOK_SECRET}
"#;

        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(trigger.spec.trigger_type, StandaloneTriggerType::HTTP);
        assert_eq!(trigger.spec.config.path, Some("/webhook/github".to_string()));
        assert!(trigger.validate().is_ok());
    }

    #[test]
    fn test_parse_pagerduty_trigger() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: pagerduty-incidents
spec:
  type: PagerDuty
  config:
    api_key: ${PAGERDUTY_API_KEY}
    service_ids:
      - P123ABC
      - P456DEF
"#;

        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(trigger.spec.trigger_type, StandaloneTriggerType::PagerDuty);
        assert!(trigger.validate().is_ok());
    }

    #[test]
    fn test_validation_errors() {
        // Empty name
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: ""
spec:
  type: Slack
  config:
    bot_token: token
"#;
        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();
        assert!(trigger.validate().is_err());

        // Missing bot_token for Slack
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: test
spec:
  type: Slack
  config: {}
"#;
        let trigger2: Trigger = serde_yaml::from_str(yaml2).unwrap();
        assert!(trigger2.validate().is_err());

        // Missing cron for Schedule
        let yaml3 = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: test
spec:
  type: Schedule
  config: {}
"#;
        let trigger3: Trigger = serde_yaml::from_str(yaml3).unwrap();
        assert!(trigger3.validate().is_err());
    }

    #[test]
    fn test_matches() {
        let yaml = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: test
spec:
  type: Slack
  config:
    bot_token: token
    channels:
      - production
    patterns:
      - kubectl
"#;

        let trigger: Trigger = serde_yaml::from_str(yaml).unwrap();

        // Matches
        assert!(trigger.matches("slack", Some("production"), None, Some("kubectl get pods")));

        // Wrong platform
        assert!(!trigger.matches("telegram", Some("production"), None, Some("kubectl get pods")));

        // Wrong channel
        assert!(!trigger.matches("slack", Some("staging"), None, Some("kubectl get pods")));

        // Pattern doesn't match
        assert!(!trigger.matches("slack", Some("production"), None, Some("hello world")));
    }

    #[test]
    fn test_match_score() {
        // Trigger with channel + pattern filter (most specific)
        let yaml1 = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: specific
spec:
  type: Slack
  config:
    bot_token: token
    channels: [production]
    patterns: [kubectl]
"#;

        // Trigger with only platform (catch-all)
        let yaml2 = r#"
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: catchall
spec:
  type: Slack
  config:
    bot_token: token
"#;

        let specific: Trigger = serde_yaml::from_str(yaml1).unwrap();
        let catchall: Trigger = serde_yaml::from_str(yaml2).unwrap();

        let score1 = specific.match_score("slack", Some("production"), None, Some("kubectl get pods"));
        let score2 = catchall.match_score("slack", Some("production"), None, Some("kubectl get pods"));

        // More specific trigger should have higher score
        assert!(score1 > score2);
    }

    #[test]
    fn test_expand_env_var() {
        std::env::set_var("TEST_TOKEN", "secret123");
        let result = expand_env_var("Bearer ${TEST_TOKEN}");
        assert_eq!(result, "Bearer secret123");
        std::env::remove_var("TEST_TOKEN");
    }
}
