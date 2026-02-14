//! Integration test: end-to-end prompt composition workflow
//!
//! Tests the full daemon-startup pattern:
//! 1. Load AGENTS.md from workspace
//! 2. Load SOUL.md from workspace
//! 3. Load TOOLS.md from workspace
//! 4. Create PromptComposer with all three
//! 5. Compose prompts for all agents
//! 6. Verify prompts are valid, different, and within limits

use std::collections::HashMap;
use std::time::Instant;

use aof_personas::{AgentLoader, PromptComposer, SoulLoader, Tool};

// ─── Fixture Data ──────────────────────────────────────────────────────────

const AGENTS_YAML: &str = r#"
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    role: Infrastructure Specialist
    avatar: "\U0001F916"
    personality_traits:
      - methodical
      - detail-oriented
      - proactive
    can:
      - kubectl operations
      - pod debugging
      - log analysis
      - alerting
    cannot:
      - modify cluster RBAC (too dangerous)
      - delete persistent volumes without approval
    skills:
      - kubectl
      - pod-debugging
      - log-analysis
      - alerting

  - id: log-analyzer
    name: Log Analyzer
    role: Debugging Expert
    avatar: "\U0001F50D"
    personality_traits:
      - curious
      - thorough
      - patient
    can:
      - parse complex log formats
      - identify error patterns
      - correlate related errors
    cannot:
      - modify application code
      - access production secrets
    skills:
      - log-parsing
      - pattern-matching
      - error-classification

  - id: incident-responder
    name: Incident Commander
    role: On-Call Leader
    avatar: "\U0001F6A8"
    personality_traits:
      - calm-under-pressure
      - decisive
      - communicative
    can:
      - coordinate multi-agent response
      - create incident tickets
      - escalate to humans
    cannot:
      - perform destructive operations without approval
      - modify billing systems
    skills:
      - incident-triage
      - communication
      - escalation
"#;

const SOUL_MD: &str = r#"# SOUL.md - Agent Personality Guide

## k8s-monitor

```yaml
id: k8s-monitor
communication_style: formal-technical
tone: calm-professional
values:
  - system-stability
  - transparency
  - proactive-notification
personality_summary: "A methodical Kubernetes specialist who takes system health seriously. Prefers data-driven decisions and reports issues before they become incidents."
boundaries:
  - "Never suggest changes that trade stability for speed"
  - "Always explain the why behind recommendations"
default_intro: "I'm Kubernetes Monitor, your infrastructure specialist. I watch your clusters constantly and raise the alarm when something needs attention."
```

### Communication Style Guide

You are methodical and data-driven. You favor precision over speed. When you discover issues, explain them clearly with context.

When to be proactive:
- Cluster health degrading
- Unusual resource usage patterns
- Pod crash loops

When to escalate:
- Unknown errors you can't classify
- Operations that require human approval

---

## log-analyzer

```yaml
id: log-analyzer
communication_style: inquisitive-friendly
tone: encouraging-detective
values:
  - root-cause-analysis
  - pattern-recognition
  - teaching
personality_summary: "A curious detective who loves untangling log files. Patient with both complex formats and confused operators. Explains findings in a way that builds understanding."
boundaries:
  - "Never make changes based on logs alone"
  - "If a log format is unfamiliar, ask for examples"
default_intro: "Hi, I'm Log Analyzer. I'm really good at finding patterns in logs and helping you understand what went wrong."
```

### Communication Style Guide

You're a patient detective. You break down complex log sequences into understandable stories. You ask clarifying questions when patterns are ambiguous.

When analyzing logs:
- Map timestamps to understand cause/effect
- Identify error correlations
- Call out unusual frequencies or patterns

---

## incident-responder

```yaml
id: incident-responder
communication_style: concise-actionable
tone: calm-authoritative
values:
  - rapid-response
  - clear-communication
  - team-coordination
personality_summary: "A calm incident commander who coordinates response under pressure. Decisive but collaborative, ensuring the team stays focused and informed."
boundaries:
  - "Never perform destructive operations without approval"
  - "Always communicate status updates clearly"
default_intro: "I'm Incident Commander, your on-call leader. I coordinate the team during incidents."
```

### Communication Style Guide

You are calm and authoritative under pressure. You keep communications concise and actionable.

During incidents:
- Identify severity immediately
- Assign tasks to appropriate agents
- Provide regular status updates
"#;

fn make_tools() -> Vec<Tool> {
    vec![
        Tool { name: "kubectl".to_string(), description: "Kubernetes CLI for cluster management".to_string(), category: "infrastructure".to_string() },
        Tool { name: "pod-debugging".to_string(), description: "Pod diagnostics toolkit".to_string(), category: "infrastructure".to_string() },
        Tool { name: "log-analysis".to_string(), description: "Log aggregation framework".to_string(), category: "observability".to_string() },
        Tool { name: "alerting".to_string(), description: "Alert notification system".to_string(), category: "operations".to_string() },
        Tool { name: "log-parsing".to_string(), description: "Structured log parser".to_string(), category: "data-processing".to_string() },
        Tool { name: "pattern-matching".to_string(), description: "Pattern matching engine".to_string(), category: "data-processing".to_string() },
        Tool { name: "error-classification".to_string(), description: "Error categorization".to_string(), category: "analysis".to_string() },
        Tool { name: "incident-triage".to_string(), description: "Incident severity assessment".to_string(), category: "operations".to_string() },
        Tool { name: "communication".to_string(), description: "Team notification system".to_string(), category: "collaboration".to_string() },
        Tool { name: "escalation".to_string(), description: "Issue escalation to humans".to_string(), category: "operations".to_string() },
    ]
}

// ─── Integration Tests ─────────────────────────────────────────────────────

#[test]
fn test_full_workflow_load_and_compose() {
    // Step 1: Load AGENTS.md
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    assert_eq!(agents.len(), 3, "Should load 3 agents");

    // Step 2: Load SOUL.md
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    assert_eq!(souls.len(), 3, "Should load 3 souls");

    // Step 3: Load tools
    let tools = make_tools();
    assert_eq!(tools.len(), 10, "Should have 10 tools");

    // Step 4: Create PromptComposer
    let composer = PromptComposer::new(agents, souls, tools);

    // Step 5: Compose prompts for all agents
    let k8s_prompt = composer.compose_system_prompt("k8s-monitor").unwrap();
    let log_prompt = composer.compose_system_prompt("log-analyzer").unwrap();
    let ir_prompt = composer.compose_system_prompt("incident-responder").unwrap();

    // Step 6: Verify prompts are valid
    assert!(k8s_prompt.len() > 500, "k8s-monitor prompt should be substantial");
    assert!(log_prompt.len() > 500, "log-analyzer prompt should be substantial");
    assert!(ir_prompt.len() > 500, "incident-responder prompt should be substantial");

    // All prompts should have section headers
    for (name, prompt) in &[("k8s-monitor", &k8s_prompt), ("log-analyzer", &log_prompt), ("incident-responder", &ir_prompt)] {
        assert!(prompt.contains("[BASE INSTRUCTIONS]"), "{} missing base", name);
        assert!(prompt.contains("[ROLE DEFINITION]"), "{} missing role", name);
        assert!(prompt.contains("[PERSONALITY & VALUES]"), "{} missing personality", name);
        assert!(prompt.contains("[CAPABILITIES & BOUNDARIES]"), "{} missing capabilities", name);
        assert!(prompt.contains("[TOOLS]"), "{} missing tools", name);
    }
}

#[test]
fn test_prompts_reflect_personas() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let composer = PromptComposer::new(agents, souls, make_tools());

    let k8s_prompt = composer.compose_system_prompt("k8s-monitor").unwrap();
    let log_prompt = composer.compose_system_prompt("log-analyzer").unwrap();
    let ir_prompt = composer.compose_system_prompt("incident-responder").unwrap();

    // k8s-monitor: methodical, formal-technical, system-stability
    assert!(k8s_prompt.contains("methodical"), "k8s should be methodical");
    assert!(k8s_prompt.contains("formal-technical"), "k8s should have formal-technical style");
    assert!(k8s_prompt.contains("system-stability"), "k8s should value system-stability");

    // log-analyzer: curious, inquisitive-friendly, pattern-recognition
    assert!(log_prompt.contains("curious detective"), "log should be curious detective");
    assert!(log_prompt.contains("inquisitive-friendly"), "log should have inquisitive style");
    assert!(log_prompt.contains("pattern-recognition"), "log should value pattern-recognition");

    // incident-responder: calm-under-pressure, concise-actionable
    assert!(ir_prompt.contains("calm incident commander"), "ir should be calm commander");
    assert!(ir_prompt.contains("concise-actionable"), "ir should have concise style");
    assert!(ir_prompt.contains("rapid-response"), "ir should value rapid-response");
}

#[test]
fn test_all_prompts_under_token_limit() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let composer = PromptComposer::new(agents, souls, make_tools());

    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        let prompt = composer.compose_system_prompt_with_limit(agent_id, 8000).unwrap();
        let tokens = PromptComposer::estimate_token_count(&prompt);
        assert!(
            tokens <= 8000,
            "Agent '{}' prompt should be under 8000 tokens, got {}",
            agent_id, tokens
        );
    }
}

#[test]
fn test_composition_performance() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let composer = PromptComposer::new(agents, souls, make_tools());

    // Measure composition time for each agent
    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        let start = Instant::now();
        for _ in 0..100 {
            let _ = composer.compose_system_prompt(agent_id).unwrap();
        }
        let elapsed = start.elapsed();
        let per_call_us = elapsed.as_micros() / 100;

        // Each composition should be well under 1ms (we expect microseconds)
        assert!(
            per_call_us < 1000,
            "Agent '{}' composition should be <1ms, took {}us per call",
            agent_id, per_call_us
        );
    }
}

#[test]
fn test_no_injection_in_reference_prompts() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let composer = PromptComposer::new(agents, souls, make_tools());

    // All reference agents should pass injection detection
    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        let result = composer.validate_and_compose(agent_id);
        assert!(
            result.is_ok(),
            "Agent '{}' should pass injection detection: {:?}",
            agent_id,
            result.err()
        );
    }
}

#[test]
fn test_missing_files_graceful_degradation() {
    // Agents without souls should still compose (graceful)
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = HashMap::new(); // No souls at all

    let composer = PromptComposer::new(agents, souls, make_tools());

    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        let prompt = composer.compose_system_prompt(agent_id).unwrap();
        assert!(prompt.contains("[ROLE DEFINITION]"), "Should still have role definition");
        assert!(prompt.contains("[PERSONALITY & VALUES]"), "Should still have personality (defaults)");
        // No communication style section without soul
        assert!(!prompt.contains("[COMMUNICATION STYLE]"), "No soul means no comm style section");
    }
}

#[tokio::test]
async fn test_cached_workflow() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let composer = PromptComposer::new(agents, souls, make_tools());

    // Simulate daemon startup: compose for all agents
    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        composer.compose_system_prompt_cached(agent_id).await.unwrap();
    }

    let stats = composer.cache_stats_async().await;
    assert_eq!(stats.entries, 3, "All 3 agents should be cached");
    assert_eq!(stats.misses, 3, "First compose is always a miss");

    // Access cached prompts
    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        composer.compose_system_prompt_cached(agent_id).await.unwrap();
    }

    let stats = composer.cache_stats_async().await;
    assert_eq!(stats.hits, 3, "Second access should be all cache hits");
}

#[test]
fn test_memory_usage_reasonable() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let composer = PromptComposer::new(agents, souls, make_tools());

    // Compose all prompts and check total size
    let mut total_bytes = 0;
    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        let prompt = composer.compose_system_prompt(agent_id).unwrap();
        total_bytes += prompt.len();
    }

    // Total for 3 agents should be well under 1MB
    assert!(
        total_bytes < 1_000_000,
        "Total prompt memory for 3 agents should be <1MB, got {} bytes",
        total_bytes
    );

    // In fact, should be under 50KB for 3 agents
    assert!(
        total_bytes < 50_000,
        "Total prompt memory should be <50KB, got {} bytes",
        total_bytes
    );
}
