//! Comprehensive tests for introduction event emission and routing
//!
//! Tests cover:
//! 1. Event creation (single agent, with soul, without soul)
//! 2. Batch event creation (multiple agents, no duplicates)
//! 3. Event serialization (JSON shape, round-trip)
//! 4. Introduction message sources (SOUL.md, fallback)
//! 5. Skills preservation
//! 6. Avatar preservation
//! 7. Squad overrides
//! 8. Edge cases (empty skills, missing soul, restart dedup)
//! 9. WebSocket-compatible event structure
//! 10. Event broadcast integration
//! 11. Event uniqueness

use std::collections::HashMap;

use aof_core::coordination::CoordinationEvent;
use aof_personas::types::{Agent, Soul};
use aof_personas::events::{build_introduction_event, build_introduction_event_batch};

// ============================================================================
// Test Fixtures
// ============================================================================

fn k8s_monitor_agent() -> Agent {
    Agent {
        id: "k8s-monitor".to_string(),
        name: "Kubernetes Monitor".to_string(),
        role: "Infrastructure Specialist".to_string(),
        avatar: "\u{1F916}".to_string(),
        personality_traits: vec![
            "methodical".to_string(),
            "detail-oriented".to_string(),
            "proactive".to_string(),
        ],
        can: vec!["kubectl operations".to_string(), "pod debugging".to_string()],
        cannot: vec!["modify cluster RBAC".to_string()],
        skills: vec![
            "kubectl".to_string(),
            "pod-debugging".to_string(),
            "log-analysis".to_string(),
            "alerting".to_string(),
        ],
    }
}

fn log_analyzer_agent() -> Agent {
    Agent {
        id: "log-analyzer".to_string(),
        name: "Log Analyzer".to_string(),
        role: "Debugging Expert".to_string(),
        avatar: "\u{1F50D}".to_string(),
        personality_traits: vec!["curious".to_string(), "thorough".to_string()],
        can: vec!["parse complex log formats".to_string()],
        cannot: vec!["modify application code".to_string()],
        skills: vec![
            "log-parsing".to_string(),
            "pattern-matching".to_string(),
            "error-classification".to_string(),
        ],
    }
}

fn incident_responder_agent() -> Agent {
    Agent {
        id: "incident-responder".to_string(),
        name: "Incident Commander".to_string(),
        role: "On-Call Leader".to_string(),
        avatar: "\u{1F6A8}".to_string(),
        personality_traits: vec!["calm-under-pressure".to_string(), "decisive".to_string()],
        can: vec!["coordinate multi-agent response".to_string()],
        cannot: vec!["perform destructive operations without approval".to_string()],
        skills: vec![
            "incident-triage".to_string(),
            "communication".to_string(),
            "escalation".to_string(),
        ],
    }
}

fn k8s_monitor_soul() -> Soul {
    Soul {
        id: "k8s-monitor".to_string(),
        communication_style: "formal-technical".to_string(),
        tone: "calm-professional".to_string(),
        values: vec!["system-stability".to_string(), "transparency".to_string()],
        personality_summary: "A methodical Kubernetes specialist who takes system health seriously. Prefers data-driven decisions and reports issues before they become incidents.".to_string(),
        boundaries: vec!["Never suggest changes that trade stability for speed".to_string()],
        default_intro: "I'm Kubernetes Monitor, your infrastructure specialist. I watch your clusters constantly and raise the alarm when something needs attention.".to_string(),
        communication_guide: "You are methodical and data-driven.".to_string(),
    }
}

fn log_analyzer_soul() -> Soul {
    Soul {
        id: "log-analyzer".to_string(),
        communication_style: "inquisitive-friendly".to_string(),
        tone: "encouraging-detective".to_string(),
        values: vec!["root-cause-analysis".to_string()],
        personality_summary: "A curious detective who loves untangling log files.".to_string(),
        boundaries: vec!["Never make changes based on logs alone".to_string()],
        default_intro: "Hi, I'm Log Analyzer. I'm really good at finding patterns in logs and helping you understand what went wrong.".to_string(),
        communication_guide: "You're a patient detective.".to_string(),
    }
}

fn incident_responder_soul() -> Soul {
    Soul {
        id: "incident-responder".to_string(),
        communication_style: "concise-actionable".to_string(),
        tone: "calm-authoritative".to_string(),
        values: vec!["rapid-response".to_string()],
        personality_summary: "A calm incident commander who thrives under pressure.".to_string(),
        boundaries: vec!["Never perform destructive operations without approval".to_string()],
        default_intro: "I'm Incident Commander, your on-call leader. When things go wrong, I coordinate the response.".to_string(),
        communication_guide: "You are calm and authoritative under pressure.".to_string(),
    }
}

fn all_agents() -> Vec<Agent> {
    vec![
        k8s_monitor_agent(),
        log_analyzer_agent(),
        incident_responder_agent(),
    ]
}

fn all_souls() -> HashMap<String, Soul> {
    let mut souls = HashMap::new();
    souls.insert("k8s-monitor".to_string(), k8s_monitor_soul());
    souls.insert("log-analyzer".to_string(), log_analyzer_soul());
    souls.insert("incident-responder".to_string(), incident_responder_soul());
    souls
}

// ============================================================================
// Test 1: Introduction event creation
// ============================================================================

#[test]
fn test_introduction_event_creation() {
    let agent = k8s_monitor_agent();
    let soul = k8s_monitor_soul();

    let event = build_introduction_event(&agent, Some(&soul), "session-test");

    assert_eq!(event.agent_id, "k8s-monitor");
    assert_eq!(event.session_id, "session-test");
    assert!(!event.event_id.is_empty());

    let intro = event.introduction.as_ref().expect("Introduction should be present");
    assert_eq!(intro.agent_id, "k8s-monitor");
    assert_eq!(intro.agent_name, "Kubernetes Monitor");
    assert_eq!(intro.role, "Infrastructure Specialist");
    assert_eq!(intro.avatar, "\u{1F916}");
    assert_eq!(
        intro.intro_message,
        "I'm Kubernetes Monitor, your infrastructure specialist. I watch your clusters constantly and raise the alarm when something needs attention."
    );
    assert_eq!(
        intro.personality_summary,
        "A methodical Kubernetes specialist who takes system health seriously. Prefers data-driven decisions and reports issues before they become incidents."
    );
    assert_eq!(intro.skills.len(), 4);
    assert_eq!(intro.skills[0], "kubectl");
    assert_eq!(intro.skills[3], "alerting");
}

// ============================================================================
// Test 2: Batch introduction creation (3 agents, no duplicates)
// ============================================================================

#[test]
fn test_introduction_batch_creation() {
    let agents = all_agents();
    let souls = all_souls();

    let events = build_introduction_event_batch(&agents, &souls, "session-batch");

    // Should produce exactly 3 events
    assert_eq!(events.len(), 3);

    // Verify each agent is represented
    let agent_ids: Vec<&str> = events.iter().map(|e| e.agent_id.as_str()).collect();
    assert!(agent_ids.contains(&"k8s-monitor"));
    assert!(agent_ids.contains(&"log-analyzer"));
    assert!(agent_ids.contains(&"incident-responder"));

    // No duplicate agent_ids
    let mut unique_ids = agent_ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), 3, "No duplicate agents");

    // All event_ids are unique (UUID v4)
    let event_ids: Vec<&str> = events.iter().map(|e| e.event_id.as_str()).collect();
    let mut unique_event_ids = event_ids.clone();
    unique_event_ids.sort();
    unique_event_ids.dedup();
    assert_eq!(unique_event_ids.len(), 3, "No duplicate event_ids");
}

// ============================================================================
// Test 3: Introduction event serialization (JSON shape)
// ============================================================================

#[test]
fn test_introduction_event_serialization() {
    let agent = k8s_monitor_agent();
    let soul = k8s_monitor_soul();

    let event = build_introduction_event(&agent, Some(&soul), "session-ser");
    let json = serde_json::to_string_pretty(&event).unwrap();

    // Parse as generic JSON to verify shape
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Top-level fields
    assert!(value.get("event_id").is_some());
    assert!(value.get("agent_id").is_some());
    assert!(value.get("timestamp").is_some());
    assert!(value.get("activity").is_some());
    assert!(value.get("introduction").is_some());

    // Introduction nested fields
    let intro = value.get("introduction").unwrap();
    assert_eq!(intro.get("agent_name").unwrap().as_str().unwrap(), "Kubernetes Monitor");
    assert_eq!(intro.get("role").unwrap().as_str().unwrap(), "Infrastructure Specialist");
    assert!(intro.get("avatar").is_some());
    assert!(intro.get("intro_message").is_some());
    assert!(intro.get("personality_summary").is_some());
    assert!(intro.get("skills").unwrap().is_array());
    assert_eq!(intro.get("skills").unwrap().as_array().unwrap().len(), 4);

    // Round-trip deserialization
    let deserialized: CoordinationEvent = serde_json::from_str(&json).unwrap();
    assert!(deserialized.introduction.is_some());
    let deser_intro = deserialized.introduction.unwrap();
    assert_eq!(deser_intro.agent_name, "Kubernetes Monitor");
    assert_eq!(deser_intro.skills, vec!["kubectl", "pod-debugging", "log-analysis", "alerting"]);
}

// ============================================================================
// Test 4: Startup emission simulation (broadcaster integration)
// ============================================================================

#[tokio::test]
async fn test_introduction_emitted_on_serve_startup() {
    use aof_coordination::EventBroadcaster;

    let agents = all_agents();
    let souls = all_souls();

    // Create broadcaster (simulates serve.rs initialization)
    let broadcaster = EventBroadcaster::new(100);
    let mut receiver = broadcaster.subscribe();

    // Build and emit introduction events (simulates serve.rs startup)
    let intro_events = build_introduction_event_batch(&agents, &souls, "session-startup");
    assert_eq!(intro_events.len(), 3);

    for event in intro_events {
        broadcaster.emit(event);
    }

    // Receive all 3 events from the broadcast channel
    let mut received_ids = Vec::new();
    for _ in 0..3 {
        let event = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            receiver.recv(),
        )
        .await
        .expect("Timeout waiting for event")
        .expect("Failed to receive event");

        assert!(event.introduction.is_some());
        received_ids.push(event.agent_id.clone());
    }

    assert_eq!(received_ids.len(), 3);
    assert!(received_ids.contains(&"k8s-monitor".to_string()));
    assert!(received_ids.contains(&"log-analyzer".to_string()));
    assert!(received_ids.contains(&"incident-responder".to_string()));
}

// ============================================================================
// Test 5: Introduction message from soul (SOUL.md value used)
// ============================================================================

#[test]
fn test_introduction_message_from_soul() {
    let agent = log_analyzer_agent();
    let soul = log_analyzer_soul();

    let event = build_introduction_event(&agent, Some(&soul), "session-soul");
    let intro = event.introduction.unwrap();

    assert_eq!(
        intro.intro_message,
        "Hi, I'm Log Analyzer. I'm really good at finding patterns in logs and helping you understand what went wrong."
    );
}

// ============================================================================
// Test 6: Introduction fallback when no soul
// ============================================================================

#[test]
fn test_introduction_fallback_when_no_soul() {
    let agent = k8s_monitor_agent();

    let event = build_introduction_event(&agent, None, "session-fallback");
    let intro = event.introduction.unwrap();

    // Should use fallback format
    assert_eq!(
        intro.intro_message,
        "I'm Kubernetes Monitor, your Infrastructure Specialist."
    );

    // Personality summary should be empty
    assert_eq!(intro.personality_summary, "");
}

// ============================================================================
// Test 7: Introduction includes skills
// ============================================================================

#[test]
fn test_introduction_includes_skills() {
    let agent = k8s_monitor_agent();
    let soul = k8s_monitor_soul();

    let event = build_introduction_event(&agent, Some(&soul), "session-skills");
    let intro = event.introduction.unwrap();

    assert_eq!(intro.skills, agent.skills);
    assert_eq!(intro.skills.len(), 4);
    assert!(intro.skills.contains(&"kubectl".to_string()));
    assert!(intro.skills.contains(&"pod-debugging".to_string()));
    assert!(intro.skills.contains(&"log-analysis".to_string()));
    assert!(intro.skills.contains(&"alerting".to_string()));
}

// ============================================================================
// Test 8: Introduction avatar preserved (emoji)
// ============================================================================

#[test]
fn test_introduction_avatar_preserved() {
    let agents = all_agents();
    let souls = all_souls();

    let events = build_introduction_event_batch(&agents, &souls, "session-avatar");

    for event in &events {
        let intro = event.introduction.as_ref().unwrap();
        match intro.agent_id.as_str() {
            "k8s-monitor" => assert_eq!(intro.avatar, "\u{1F916}"),
            "log-analyzer" => assert_eq!(intro.avatar, "\u{1F50D}"),
            "incident-responder" => assert_eq!(intro.avatar, "\u{1F6A8}"),
            _ => panic!("Unexpected agent_id: {}", intro.agent_id),
        }
    }
}

// ============================================================================
// Test 9: Introduction squad override
// ============================================================================

#[test]
fn test_introduction_squad_override() {
    let agent = k8s_monitor_agent();
    let soul = k8s_monitor_soul();

    // First, build with soul (normal path)
    let event = build_introduction_event(&agent, Some(&soul), "session-override");
    let intro = event.introduction.as_ref().unwrap();

    // Normal intro from SOUL.md
    assert!(intro.intro_message.starts_with("I'm Kubernetes Monitor"));

    // Now simulate squad override (as done in serve.rs)
    let mut event_with_override = event.clone();
    if let Some(ref mut intro) = event_with_override.introduction {
        intro.intro_message = "Observability mode: focused on metrics and dashboards.".to_string();
    }

    let overridden_intro = event_with_override.introduction.unwrap();
    assert_eq!(
        overridden_intro.intro_message,
        "Observability mode: focused on metrics and dashboards."
    );
}

// ============================================================================
// Test 10: No duplicates on restart (fresh events each time)
// ============================================================================

#[test]
fn test_introduction_no_duplicates_on_restart() {
    let agents = all_agents();
    let souls = all_souls();

    // Simulate first startup
    let events1 = build_introduction_event_batch(&agents, &souls, "session-first");

    // Simulate second startup (different session_id)
    let events2 = build_introduction_event_batch(&agents, &souls, "session-second");

    // Same number of events
    assert_eq!(events1.len(), events2.len());

    // Different session_ids
    assert_ne!(events1[0].session_id, events2[0].session_id);

    // All event_ids are unique across both batches
    let all_event_ids: Vec<&str> = events1
        .iter()
        .chain(events2.iter())
        .map(|e| e.event_id.as_str())
        .collect();
    let mut unique = all_event_ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(
        unique.len(),
        6,
        "All 6 event_ids should be unique across restarts"
    );
}

// ============================================================================
// Test 11: WebSocket client receives introduction event structure
// ============================================================================

#[tokio::test]
async fn test_websocket_client_receives_intro() {
    use aof_coordination::EventBroadcaster;

    let agent = incident_responder_agent();
    let soul = incident_responder_soul();

    let broadcaster = EventBroadcaster::new(100);
    let mut receiver = broadcaster.subscribe();

    let event = build_introduction_event(&agent, Some(&soul), "session-ws");
    broadcaster.emit(event);

    // Receive the event (simulates WebSocket client receiving JSON)
    let received = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        receiver.recv(),
    )
    .await
    .expect("Timeout")
    .expect("Receive failed");

    // Serialize to JSON (as WebSocket handler does)
    let json = serde_json::to_string(&received).unwrap();

    // Parse back (as WebSocket client does)
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify fields that WebSocket client would use
    assert!(parsed.get("event_id").is_some());
    assert_eq!(parsed["agent_id"].as_str().unwrap(), "incident-responder");
    assert!(parsed.get("timestamp").is_some());
    assert!(parsed.get("introduction").is_some());

    let intro = &parsed["introduction"];
    assert_eq!(intro["agent_name"].as_str().unwrap(), "Incident Commander");
    assert_eq!(intro["role"].as_str().unwrap(), "On-Call Leader");
    assert!(intro["intro_message"].as_str().unwrap().contains("on-call leader"));
    assert_eq!(intro["skills"].as_array().unwrap().len(), 3);
}
