//! End-to-end integration test: full persona workflow
//!
//! This test validates the entire persona system pipeline from workspace files
//! to metrics computation, simulating the complete daemon startup flow:
//!
//! 1. Load AGENTS.md -> parse 3 agents
//! 2. Load SOUL.md -> parse 3 personalities
//! 3. Create PromptComposer with agents + souls + tools
//! 4. Compose system prompts for all agents (verify distinct prompts)
//! 5. Build introduction event batch (verify 3 events)
//! 6. Emit intro events via broadcast channel (verify subscriber receipt)
//! 7. Verify composed prompts reflect personality cues
//! 8. Feed events into ReliabilityCache (verify metrics computation)
//! 9. Verify metrics badge data (uptime %, success rate)
//! 10. Validate full system under token limits and performance constraints
//!
//! This test serves as the definitive integration validation for Phase 5
//! (Agent Personas) and can be used as a reference for users adding new agents.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use aof_core::activity::{ActivityEvent, ActivityType};
use aof_core::coordination::CoordinationEvent;
use aof_personas::{
    build_introduction_event, build_introduction_event_batch, compute_agent_metrics,
    validate_personas, AgentLoader, PromptComposer, ReliabilityCache,
    SoulLoader, Tool,
};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Fixture Data: Reference workspace files
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Reference AGENTS.md with 3 agents: k8s-monitor, log-analyzer, incident-responder
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

/// Reference SOUL.md with personality guidance for all 3 agents
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

You are methodical and data-driven. You favor precision over speed. When you discover issues, explain them clearly with context (affected resources, impact scope, potential causes). Use structured output (tables, lists, JSON when appropriate).

When to be proactive:
- Cluster health degrading
- Unusual resource usage patterns
- Pod crash loops
- Node pressure (memory, disk)

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

/// Reference tool definitions (from TOOLS.md equivalent)
fn make_reference_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "kubectl".to_string(),
            description: "Kubernetes CLI for cluster management".to_string(),
            category: "infrastructure".to_string(),
        },
        Tool {
            name: "pod-debugging".to_string(),
            description: "Pod diagnostics toolkit".to_string(),
            category: "infrastructure".to_string(),
        },
        Tool {
            name: "log-analysis".to_string(),
            description: "Log aggregation framework".to_string(),
            category: "observability".to_string(),
        },
        Tool {
            name: "alerting".to_string(),
            description: "Alert notification system".to_string(),
            category: "operations".to_string(),
        },
        Tool {
            name: "log-parsing".to_string(),
            description: "Structured log parser".to_string(),
            category: "data-processing".to_string(),
        },
        Tool {
            name: "pattern-matching".to_string(),
            description: "Pattern matching engine".to_string(),
            category: "data-processing".to_string(),
        },
        Tool {
            name: "error-classification".to_string(),
            description: "Error categorization system".to_string(),
            category: "analysis".to_string(),
        },
        Tool {
            name: "incident-triage".to_string(),
            description: "Incident severity assessment".to_string(),
            category: "operations".to_string(),
        },
        Tool {
            name: "communication".to_string(),
            description: "Team notification system".to_string(),
            category: "collaboration".to_string(),
        },
        Tool {
            name: "escalation".to_string(),
            description: "Issue escalation to humans".to_string(),
            category: "operations".to_string(),
        },
    ]
}

/// Helper: create a CoordinationEvent with a specific activity type
fn make_event(agent_id: &str, activity_type: ActivityType, message: &str) -> CoordinationEvent {
    let activity = ActivityEvent::new(activity_type, message);
    CoordinationEvent::from_activity(activity, agent_id, "e2e-session")
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 1-2: Load workspace files
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn step_01_load_agents_md() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();

    // Must load exactly 3 agents
    assert_eq!(agents.len(), 3, "AGENTS.md should define exactly 3 agents");

    // Verify agent IDs
    let ids: Vec<&str> = agents.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"k8s-monitor"), "Missing k8s-monitor");
    assert!(ids.contains(&"log-analyzer"), "Missing log-analyzer");
    assert!(ids.contains(&"incident-responder"), "Missing incident-responder");

    // Verify each agent has required fields
    for agent in &agents {
        assert!(!agent.id.is_empty(), "Agent ID must not be empty");
        assert!(!agent.name.is_empty(), "Agent name must not be empty");
        assert!(!agent.role.is_empty(), "Agent role must not be empty");
        assert!(!agent.avatar.is_empty(), "Agent avatar must not be empty");
        assert!(
            !agent.personality_traits.is_empty(),
            "Agent {} must have personality traits",
            agent.id
        );
        assert!(
            !agent.can.is_empty(),
            "Agent {} must have capabilities",
            agent.id
        );
        assert!(
            !agent.cannot.is_empty(),
            "Agent {} must have boundaries",
            agent.id
        );
        assert!(
            !agent.skills.is_empty(),
            "Agent {} must have skills",
            agent.id
        );
    }
}

#[test]
fn step_02_load_soul_md() {
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();

    // Must load exactly 3 souls
    assert_eq!(souls.len(), 3, "SOUL.md should define exactly 3 personalities");

    // Verify soul IDs match expected agents
    assert!(souls.contains_key("k8s-monitor"), "Missing k8s-monitor soul");
    assert!(souls.contains_key("log-analyzer"), "Missing log-analyzer soul");
    assert!(
        souls.contains_key("incident-responder"),
        "Missing incident-responder soul"
    );

    // Verify each soul has required fields
    for (id, soul) in &souls {
        assert!(!soul.communication_style.is_empty(), "{}: missing comm style", id);
        assert!(!soul.tone.is_empty(), "{}: missing tone", id);
        assert!(!soul.values.is_empty(), "{}: missing values", id);
        assert!(
            !soul.personality_summary.is_empty(),
            "{}: missing personality summary",
            id
        );
        assert!(!soul.boundaries.is_empty(), "{}: missing boundaries", id);
        assert!(!soul.default_intro.is_empty(), "{}: missing intro", id);
        assert!(
            !soul.communication_guide.is_empty(),
            "{}: missing communication guide",
            id
        );
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 3: Validate personas (cross-reference integrity)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn step_03_validate_personas_cross_reference() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();

    // Full validation (agents + souls + cross-references)
    let result = validate_personas(&agents, &souls);
    assert!(
        result.is_ok(),
        "Persona validation should pass: {:?}",
        result.err()
    );

    // Verify every soul ID matches an agent ID
    let agent_ids: HashSet<&str> = agents.iter().map(|a| a.id.as_str()).collect();
    for soul_id in souls.keys() {
        assert!(
            agent_ids.contains(soul_id.as_str()),
            "Soul '{}' has no matching agent",
            soul_id
        );
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 4: Compose system prompts for all agents
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn step_04_compose_system_prompts_all_agents() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let tools = make_reference_tools();

    let composer = PromptComposer::new(agents, souls, tools);

    let agent_ids = ["k8s-monitor", "log-analyzer", "incident-responder"];
    let mut prompts: HashMap<&str, String> = HashMap::new();

    for agent_id in &agent_ids {
        let prompt = composer.compose_system_prompt(agent_id).unwrap();

        // Every prompt must contain all 7 instruction layers
        assert!(
            prompt.contains("[BASE INSTRUCTIONS]"),
            "{}: missing base layer",
            agent_id
        );
        assert!(
            prompt.contains("[ROLE DEFINITION]"),
            "{}: missing role layer",
            agent_id
        );
        assert!(
            prompt.contains("[PERSONALITY & VALUES]"),
            "{}: missing personality layer",
            agent_id
        );
        assert!(
            prompt.contains("[COMMUNICATION STYLE]"),
            "{}: missing comm style layer",
            agent_id
        );
        assert!(
            prompt.contains("[CAPABILITIES & BOUNDARIES]"),
            "{}: missing capabilities layer",
            agent_id
        );
        assert!(
            prompt.contains("[TOOLS]"),
            "{}: missing tools layer",
            agent_id
        );
        assert!(
            prompt.contains("[BEHAVIORAL RULES]"),
            "{}: missing behavioral rules layer",
            agent_id
        );

        // Prompt should be substantial (>500 chars)
        assert!(
            prompt.len() > 500,
            "{}: prompt too short ({} chars)",
            agent_id,
            prompt.len()
        );

        prompts.insert(agent_id, prompt);
    }

    // All 3 prompts must be DIFFERENT (distinct personas produce distinct prompts)
    let prompt_values: Vec<&String> = prompts.values().collect();
    for i in 0..prompt_values.len() {
        for j in (i + 1)..prompt_values.len() {
            assert_ne!(
                prompt_values[i], prompt_values[j],
                "Prompts should be unique per agent"
            );
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 5: Build introduction event batch
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn step_05_build_introduction_events() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();

    // Build batch of introduction events
    let events = build_introduction_event_batch(&agents, &souls, "e2e-session");

    // Must produce exactly 3 events (one per agent)
    assert_eq!(events.len(), 3, "Should create 3 introduction events");

    // Verify all events have unique event_ids
    let event_ids: HashSet<&str> = events.iter().map(|e| e.event_id.as_str()).collect();
    assert_eq!(event_ids.len(), 3, "All event IDs must be unique");

    // Verify each event has correct session and introduction data
    for event in &events {
        assert_eq!(event.session_id, "e2e-session", "Session ID mismatch");
        assert!(event.introduction.is_some(), "Introduction data missing");

        let intro = event.introduction.as_ref().unwrap();
        assert!(!intro.agent_name.is_empty(), "Agent name empty");
        assert!(!intro.role.is_empty(), "Role empty");
        assert!(!intro.avatar.is_empty(), "Avatar empty");
        assert!(!intro.intro_message.is_empty(), "Intro message empty");
    }

    // Verify k8s-monitor gets its SOUL.md intro message
    let k8s_event = events.iter().find(|e| e.agent_id == "k8s-monitor").unwrap();
    let k8s_intro = k8s_event.introduction.as_ref().unwrap();
    assert!(
        k8s_intro.intro_message.contains("infrastructure specialist"),
        "k8s-monitor intro should mention infrastructure specialist"
    );
    assert_eq!(
        k8s_intro.personality_summary,
        "A methodical Kubernetes specialist who takes system health seriously. Prefers data-driven decisions and reports issues before they become incidents."
    );

    // Verify log-analyzer gets its SOUL.md intro
    let log_event = events.iter().find(|e| e.agent_id == "log-analyzer").unwrap();
    let log_intro = log_event.introduction.as_ref().unwrap();
    assert!(
        log_intro.intro_message.contains("patterns in logs"),
        "log-analyzer intro should mention patterns in logs"
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 6: Emit events via broadcast channel (subscriber receipt)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn step_06_emit_intro_events_via_broadcast() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();

    let events = build_introduction_event_batch(&agents, &souls, "e2e-session");

    // Create broadcast channel (simulates daemon EventBroadcaster)
    let (tx, mut rx) = tokio::sync::broadcast::channel::<CoordinationEvent>(32);

    // Emit all introduction events
    for event in &events {
        tx.send(event.clone()).unwrap();
    }

    // Verify subscriber receives all 3 events
    let mut received = Vec::new();
    for _ in 0..3 {
        let event = rx.recv().await.unwrap();
        received.push(event);
    }

    assert_eq!(received.len(), 3, "Subscriber should receive all 3 events");

    // Verify received events match emitted events
    let received_ids: HashSet<String> = received.iter().map(|e| e.agent_id.clone()).collect();
    assert!(received_ids.contains("k8s-monitor"));
    assert!(received_ids.contains("log-analyzer"));
    assert!(received_ids.contains("incident-responder"));

    // All received events should have introduction data
    for event in &received {
        assert!(
            event.introduction.is_some(),
            "Received event for {} missing introduction",
            event.agent_id
        );
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 7: Verify prompts reflect personality cues
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn step_07_prompts_reflect_personality_cues() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let tools = make_reference_tools();
    let composer = PromptComposer::new(agents, souls, tools);

    // k8s-monitor: methodical, formal-technical, data-driven
    let k8s_prompt = composer.compose_system_prompt("k8s-monitor").unwrap();
    assert!(
        k8s_prompt.contains("methodical"),
        "k8s prompt should contain 'methodical'"
    );
    assert!(
        k8s_prompt.contains("data-driven"),
        "k8s prompt should contain 'data-driven'"
    );
    assert!(
        k8s_prompt.contains("formal-technical"),
        "k8s prompt should have formal-technical style"
    );
    assert!(
        k8s_prompt.contains("calm-professional"),
        "k8s prompt should have calm-professional tone"
    );
    assert!(
        k8s_prompt.contains("system-stability"),
        "k8s prompt should value system-stability"
    );
    assert!(
        k8s_prompt.contains("kubectl"),
        "k8s prompt should list kubectl tool"
    );
    assert!(
        k8s_prompt.contains("Infrastructure Specialist"),
        "k8s prompt should show role"
    );

    // log-analyzer: curious, inquisitive-friendly, detective
    let log_prompt = composer.compose_system_prompt("log-analyzer").unwrap();
    assert!(
        log_prompt.contains("curious detective"),
        "log prompt should contain 'curious detective'"
    );
    assert!(
        log_prompt.contains("inquisitive-friendly"),
        "log prompt should have inquisitive-friendly style"
    );
    assert!(
        log_prompt.contains("encouraging-detective"),
        "log prompt should have encouraging-detective tone"
    );
    assert!(
        log_prompt.contains("pattern-recognition"),
        "log prompt should value pattern-recognition"
    );
    assert!(
        log_prompt.contains("log-parsing"),
        "log prompt should list log-parsing tool"
    );

    // incident-responder: calm, concise-actionable, decisive
    let ir_prompt = composer.compose_system_prompt("incident-responder").unwrap();
    assert!(
        ir_prompt.contains("calm incident commander"),
        "ir prompt should contain 'calm incident commander'"
    );
    assert!(
        ir_prompt.contains("concise-actionable"),
        "ir prompt should have concise-actionable style"
    );
    assert!(
        ir_prompt.contains("calm-authoritative"),
        "ir prompt should have calm-authoritative tone"
    );
    assert!(
        ir_prompt.contains("rapid-response"),
        "ir prompt should value rapid-response"
    );
    assert!(
        ir_prompt.contains("incident-triage"),
        "ir prompt should list incident-triage tool"
    );

    // Verify different personas produce different communication guidance
    assert_ne!(
        k8s_prompt, log_prompt,
        "k8s and log prompts should differ"
    );
    assert_ne!(
        log_prompt, ir_prompt,
        "log and ir prompts should differ"
    );
    assert_ne!(
        k8s_prompt, ir_prompt,
        "k8s and ir prompts should differ"
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 8: Feed events into ReliabilityCache
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn step_08_reliability_cache_event_pipeline() {
    let cache = ReliabilityCache::default_capacity();

    // Simulate k8s-monitor: 9 successes, 1 error (90% uptime, 90% success)
    for i in 0..9 {
        let event = make_event("k8s-monitor", ActivityType::Completed, &format!("task-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }
    let error_event = make_event("k8s-monitor", ActivityType::Error, "pod-crash");
    cache.update_with_event(&error_event).await.unwrap();

    // Simulate log-analyzer: 10 successes, 0 errors (100% uptime)
    for i in 0..10 {
        let event = make_event(
            "log-analyzer",
            ActivityType::Completed,
            &format!("analysis-{}", i),
        );
        cache.update_with_event(&event).await.unwrap();
    }

    // Simulate incident-responder: 8 successes, 2 errors (80% uptime)
    for i in 0..8 {
        let event = make_event(
            "incident-responder",
            ActivityType::Completed,
            &format!("incident-{}", i),
        );
        cache.update_with_event(&event).await.unwrap();
    }
    for i in 0..2 {
        let event = make_event(
            "incident-responder",
            ActivityType::Error,
            &format!("failure-{}", i),
        );
        cache.update_with_event(&event).await.unwrap();
    }

    // Verify k8s-monitor metrics
    let k8s_metrics = cache.get_metrics("k8s-monitor").await.unwrap();
    assert_eq!(k8s_metrics.event_count, 10);
    assert!((k8s_metrics.uptime_percent.unwrap() - 90.0).abs() < 0.1);
    assert!((k8s_metrics.success_rate.unwrap() - 90.0).abs() < 0.1);
    assert!(k8s_metrics.last_error.is_some(), "Should have last_error");

    // Verify log-analyzer metrics (perfect)
    let log_metrics = cache.get_metrics("log-analyzer").await.unwrap();
    assert_eq!(log_metrics.event_count, 10);
    assert!((log_metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);
    assert!((log_metrics.success_rate.unwrap() - 100.0).abs() < 0.1);
    assert!(log_metrics.last_error.is_none());

    // Verify incident-responder metrics
    let ir_metrics = cache.get_metrics("incident-responder").await.unwrap();
    assert_eq!(ir_metrics.event_count, 10);
    assert!((ir_metrics.uptime_percent.unwrap() - 80.0).abs() < 0.1);
    assert!((ir_metrics.success_rate.unwrap() - 80.0).abs() < 0.1);
    assert!(ir_metrics.last_error.is_some());

    // Verify cache version incremented for every event
    assert_eq!(cache.version(), 30, "30 events = 30 version increments");

    // Verify total event count
    assert_eq!(cache.event_count().await, 30, "Should store all 30 events");
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 9: Verify metrics badge rendering data
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn step_09_metrics_badge_data() {
    // Simulate events for direct metric computation
    let mut events = Vec::new();
    for i in 0..15 {
        events.push(make_event(
            "k8s-monitor",
            ActivityType::Completed,
            &format!("task-{}", i),
        ));
    }
    events.push(make_event("k8s-monitor", ActivityType::Error, "error-1"));

    let metrics = compute_agent_metrics("k8s-monitor", &events);

    // Uptime should be > 0 (specifically 93.75%)
    assert!(
        metrics.uptime_percent.is_some(),
        "Should compute uptime with 16 events"
    );
    let uptime = metrics.uptime_percent.unwrap();
    assert!(uptime > 90.0, "Uptime should be >90%: {}", uptime);
    assert!(uptime < 100.0, "Uptime should be <100% with errors: {}", uptime);

    // Success rate should be > 0 (specifically 93.75%)
    assert!(
        metrics.success_rate.is_some(),
        "Should compute success rate with 16 events"
    );
    let success = metrics.success_rate.unwrap();
    assert!(success > 90.0, "Success rate should be >90%: {}", success);

    // Badge color logic (verified by UI component):
    // >= 95% -> green, >= 80% -> yellow, < 80% -> red
    // With 93.75%, badge would be yellow
    assert!(
        uptime >= 80.0 && uptime < 95.0,
        "This metric should map to yellow badge"
    );

    // Verify serialization (API response shape)
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("\"uptime_percent\""));
    assert!(json.contains("\"success_rate\""));
    assert!(json.contains("\"event_count\":16"));
    assert!(json.contains("\"agent_id\":\"k8s-monitor\""));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Step 10: Full workflow performance and token budget
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn step_10_full_workflow_performance() {
    let start = Instant::now();

    // 1. Load files
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();
    let tools = make_reference_tools();

    // 2. Validate
    validate_personas(&agents, &souls).unwrap();

    // 3. Compose prompts
    let composer = PromptComposer::new(agents.clone(), souls.clone(), tools);
    for agent_id in &["k8s-monitor", "log-analyzer", "incident-responder"] {
        let prompt = composer
            .compose_system_prompt_with_limit(agent_id, 8000)
            .unwrap();
        let tokens = PromptComposer::estimate_token_count(&prompt);
        assert!(
            tokens <= 8000,
            "{}: prompt exceeds token limit ({} tokens)",
            agent_id,
            tokens
        );
    }

    // 4. Build introduction events
    let events = build_introduction_event_batch(&agents, &souls, "e2e-session");
    assert_eq!(events.len(), 3);

    // 5. Broadcast events
    let (tx, mut rx) = tokio::sync::broadcast::channel::<CoordinationEvent>(32);
    for event in &events {
        tx.send(event.clone()).unwrap();
    }

    // 6. Receive events
    let mut received = Vec::new();
    for _ in 0..3 {
        received.push(rx.recv().await.unwrap());
    }

    // 7. Feed into ReliabilityCache
    let cache = ReliabilityCache::default_capacity();
    for i in 0..10 {
        let event = make_event("k8s-monitor", ActivityType::Completed, &format!("t-{}", i));
        cache.update_with_event(&event).await.unwrap();
    }

    // 8. Get metrics
    let metrics = cache.get_metrics("k8s-monitor").await.unwrap();
    assert!(metrics.uptime_percent.is_some());

    // 9. Cached composition
    let prompt = composer
        .compose_system_prompt_cached("k8s-monitor")
        .await
        .unwrap();
    assert!(!prompt.is_empty());

    let elapsed = start.elapsed();

    // Total workflow must complete in <5 seconds
    assert!(
        elapsed.as_secs() < 5,
        "Full persona workflow should complete in <5s, took {:?}",
        elapsed
    );

    // In practice it should be much faster (milliseconds)
    assert!(
        elapsed.as_millis() < 500,
        "Full persona workflow should be <500ms, took {:?}",
        elapsed
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Comprehensive single-test: full persona workflow integration
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_full_persona_workflow_integration() {
    // This single test validates the entire persona system end-to-end.
    // It simulates a complete daemon startup sequence.

    let start = Instant::now();

    // ── Stage 1: Load workspace files ────────────────────────────────────────

    let agents = AgentLoader::load_from_str(AGENTS_YAML)
        .expect("AGENTS.md should parse successfully");
    assert_eq!(agents.len(), 3, "Should load 3 agents from AGENTS.md");

    let souls = SoulLoader::load_from_str(SOUL_MD)
        .expect("SOUL.md should parse successfully");
    assert_eq!(souls.len(), 3, "Should load 3 souls from SOUL.md");

    // ── Stage 2: Validate workspace data ─────────────────────────────────────

    validate_personas(&agents, &souls)
        .expect("Persona validation should pass for reference agents");

    // ── Stage 3: Initialize broadcast channel (daemon EventBroadcaster) ──────

    let (event_tx, _rx) = tokio::sync::broadcast::channel::<CoordinationEvent>(64);

    // ── Stage 4: Initialize ReliabilityCache ─────────────────────────────────

    let reliability_cache = Arc::new(ReliabilityCache::default_capacity());

    // ── Stage 5: Create PromptComposer ───────────────────────────────────────

    let tools = make_reference_tools();
    let composer = PromptComposer::new(agents.clone(), souls.clone(), tools);

    // Compose and validate prompts for all agents
    let mut all_prompts: HashMap<String, String> = HashMap::new();
    for agent in &agents {
        let prompt = composer
            .validate_and_compose(&agent.id)
            .expect(&format!("Prompt for {} should pass validation", agent.id));

        // Verify 7-layer structure
        assert!(prompt.contains("[BASE INSTRUCTIONS]"));
        assert!(prompt.contains("[ROLE DEFINITION]"));
        assert!(prompt.contains("[PERSONALITY & VALUES]"));
        assert!(prompt.contains("[COMMUNICATION STYLE]"));
        assert!(prompt.contains("[CAPABILITIES & BOUNDARIES]"));
        assert!(prompt.contains("[TOOLS]"));
        assert!(prompt.contains("[BEHAVIORAL RULES]"));

        // Verify token budget
        let tokens = PromptComposer::estimate_token_count(&prompt);
        assert!(
            tokens <= 8000,
            "{}: prompt exceeds default 8000 token limit ({} tokens)",
            agent.id,
            tokens
        );

        all_prompts.insert(agent.id.clone(), prompt);
    }

    // All prompts must be distinct
    let unique_prompts: HashSet<&String> = all_prompts.values().collect();
    assert_eq!(
        unique_prompts.len(),
        3,
        "All 3 agents should produce distinct prompts"
    );

    // ── Stage 6: Build and emit introduction events ──────────────────────────

    let intro_events = build_introduction_event_batch(&agents, &souls, "e2e-integration");
    assert_eq!(intro_events.len(), 3, "Should build 3 introduction events");

    // Subscribe before sending
    let mut subscriber = event_tx.subscribe();

    // Emit introduction events (simulates daemon startup)
    for event in &intro_events {
        event_tx.send(event.clone()).unwrap();
    }

    // Verify subscriber receives all 3 events
    let mut received_intros = Vec::new();
    for _ in 0..3 {
        let event = subscriber.recv().await.unwrap();
        assert!(event.introduction.is_some());
        received_intros.push(event);
    }
    assert_eq!(received_intros.len(), 3);

    // ── Stage 7: Simulate agent execution (feed activity events) ─────────────

    // k8s-monitor: mostly successful (9 success, 1 error)
    for i in 0..9 {
        let event = make_event("k8s-monitor", ActivityType::Completed, &format!("kubectl-{}", i));
        reliability_cache.update_with_event(&event).await.unwrap();
    }
    reliability_cache
        .update_with_event(&make_event("k8s-monitor", ActivityType::Error, "pod-oom"))
        .await
        .unwrap();

    // log-analyzer: perfect record (10 success, 0 errors)
    for i in 0..10 {
        let event = make_event(
            "log-analyzer",
            ActivityType::Completed,
            &format!("log-scan-{}", i),
        );
        reliability_cache.update_with_event(&event).await.unwrap();
    }

    // incident-responder: some failures (8 success, 2 errors)
    for i in 0..8 {
        let event = make_event(
            "incident-responder",
            ActivityType::Completed,
            &format!("incident-{}", i),
        );
        reliability_cache.update_with_event(&event).await.unwrap();
    }
    reliability_cache
        .update_with_event(&make_event(
            "incident-responder",
            ActivityType::Error,
            "escalation-fail-1",
        ))
        .await
        .unwrap();
    reliability_cache
        .update_with_event(&make_event(
            "incident-responder",
            ActivityType::Error,
            "escalation-fail-2",
        ))
        .await
        .unwrap();

    // ── Stage 8: Verify metrics for all agents ───────────────────────────────

    let k8s_metrics = reliability_cache
        .get_metrics("k8s-monitor")
        .await
        .expect("k8s-monitor should have metrics");
    assert_eq!(k8s_metrics.event_count, 10);
    assert!(
        (k8s_metrics.uptime_percent.unwrap() - 90.0).abs() < 0.1,
        "k8s uptime should be 90%"
    );
    assert!(
        (k8s_metrics.success_rate.unwrap() - 90.0).abs() < 0.1,
        "k8s success should be 90%"
    );

    let log_metrics = reliability_cache
        .get_metrics("log-analyzer")
        .await
        .expect("log-analyzer should have metrics");
    assert_eq!(log_metrics.event_count, 10);
    assert!(
        (log_metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1,
        "log uptime should be 100%"
    );

    let ir_metrics = reliability_cache
        .get_metrics("incident-responder")
        .await
        .expect("incident-responder should have metrics");
    assert_eq!(ir_metrics.event_count, 10);
    assert!(
        (ir_metrics.uptime_percent.unwrap() - 80.0).abs() < 0.1,
        "ir uptime should be 80%"
    );

    // ── Stage 9: Verify prompt-to-personality coherence ──────────────────────

    // k8s-monitor prompt should reflect methodical, data-driven personality
    let k8s_prompt = &all_prompts["k8s-monitor"];
    assert!(k8s_prompt.contains("methodical"));
    assert!(k8s_prompt.contains("data-driven"));
    assert!(k8s_prompt.contains("system-stability"));
    assert!(k8s_prompt.contains("kubectl"));

    // log-analyzer prompt should reflect detective personality
    let log_prompt = &all_prompts["log-analyzer"];
    assert!(log_prompt.contains("curious detective"));
    assert!(log_prompt.contains("pattern-recognition"));
    assert!(log_prompt.contains("log-parsing"));

    // incident-responder prompt should reflect calm leadership
    let ir_prompt = &all_prompts["incident-responder"];
    assert!(ir_prompt.contains("calm incident commander"));
    assert!(ir_prompt.contains("rapid-response"));
    assert!(ir_prompt.contains("incident-triage"));

    // ── Stage 10: Validate cache performance ─────────────────────────────────

    // Compose cached prompt (should be fast)
    let cached_start = Instant::now();
    let _cached_prompt = composer
        .compose_system_prompt_cached("k8s-monitor")
        .await
        .unwrap();
    let cached_elapsed = cached_start.elapsed();
    assert!(
        cached_elapsed.as_millis() < 10,
        "Cached prompt access should be <10ms"
    );

    // Second access is cache hit
    let _cached_again = composer
        .compose_system_prompt_cached("k8s-monitor")
        .await
        .unwrap();
    let stats = composer.cache_stats_async().await;
    assert!(stats.hits >= 1, "Should have cache hits");

    // ── Final: Total workflow time ───────────────────────────────────────────

    let total_elapsed = start.elapsed();
    assert!(
        total_elapsed.as_secs() < 5,
        "Full E2E workflow must complete in <5s, took {:?}",
        total_elapsed
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Edge case: Graceful degradation (missing SOUL.md)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_graceful_degradation_no_souls() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let empty_souls: HashMap<String, _> = HashMap::new();

    // Should still compose prompts (using defaults)
    let composer = PromptComposer::new(agents.clone(), empty_souls.clone(), make_reference_tools());

    for agent in &agents {
        let prompt = composer.compose_system_prompt(&agent.id).unwrap();
        assert!(prompt.contains("[ROLE DEFINITION]"), "Role should be present");
        assert!(
            prompt.contains("[PERSONALITY & VALUES]"),
            "Default personality should be present"
        );
        // No communication style section without souls
        assert!(
            !prompt.contains("[COMMUNICATION STYLE]"),
            "No comm style without souls"
        );
    }

    // Introduction events should use fallback messages
    let events = build_introduction_event_batch(&agents, &empty_souls, "fallback-session");
    assert_eq!(events.len(), 3);

    for event in &events {
        let intro = event.introduction.as_ref().unwrap();
        // Fallback intro should contain agent name and role
        assert!(
            intro.intro_message.contains(&intro.agent_name),
            "Fallback intro should contain agent name"
        );
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Edge case: Concurrent cache access
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_concurrent_metric_reads_during_updates() {
    let cache = Arc::new(ReliabilityCache::default_capacity());

    // Writer: continuously feed events
    let writer_cache = Arc::clone(&cache);
    let writer = tokio::spawn(async move {
        for i in 0..20 {
            let event = make_event("k8s-monitor", ActivityType::Completed, &format!("t-{}", i));
            writer_cache.update_with_event(&event).await.unwrap();
        }
    });

    // Readers: concurrently read metrics
    let mut readers = Vec::new();
    for _ in 0..5 {
        let reader_cache = Arc::clone(&cache);
        readers.push(tokio::spawn(async move {
            // Small delay to ensure some events are written
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            reader_cache.get_metrics("k8s-monitor").await
        }));
    }

    writer.await.unwrap();

    for reader in readers {
        // Readers should complete without panic or deadlock
        let _result = reader.await.unwrap();
        // Result may be None (if read before any writes) or Some (if after writes)
    }

    // Final state: 20 events for k8s-monitor
    let final_metrics = cache.get_metrics("k8s-monitor").await.unwrap();
    assert_eq!(final_metrics.event_count, 20);
    assert!((final_metrics.uptime_percent.unwrap() - 100.0).abs() < 0.1);
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Edge case: Introduction event serialization roundtrip
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_introduction_event_json_roundtrip() {
    let agents = AgentLoader::load_from_str(AGENTS_YAML).unwrap();
    let souls = SoulLoader::load_from_str(SOUL_MD).unwrap();

    let events = build_introduction_event_batch(&agents, &souls, "serial-session");

    for event in &events {
        // Serialize to JSON
        let json = serde_json::to_string_pretty(event)
            .expect("Event should serialize to JSON");

        // Verify JSON structure
        assert!(json.contains("\"agent_id\""));
        assert!(json.contains("\"session_id\""));
        assert!(json.contains("\"event_id\""));
        assert!(json.contains("\"introduction\""));

        // Deserialize back
        let deserialized: CoordinationEvent = serde_json::from_str(&json)
            .expect("Event should deserialize from JSON");

        // Verify roundtrip fidelity
        assert_eq!(deserialized.agent_id, event.agent_id);
        assert_eq!(deserialized.session_id, event.session_id);
        assert!(deserialized.introduction.is_some());

        let orig_intro = event.introduction.as_ref().unwrap();
        let deser_intro = deserialized.introduction.as_ref().unwrap();
        assert_eq!(deser_intro.agent_name, orig_intro.agent_name);
        assert_eq!(deser_intro.role, orig_intro.role);
        assert_eq!(deser_intro.avatar, orig_intro.avatar);
        assert_eq!(deser_intro.intro_message, orig_intro.intro_message);
        assert_eq!(deser_intro.skills, orig_intro.skills);
    }
}
