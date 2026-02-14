//! Introduction event builder for agent personas
//!
//! Constructs CoordinationEvents with AgentIntroduction data from
//! loaded Agent (AGENTS.md) and Soul (SOUL.md) workspace types.
//! Events are emitted at daemon startup and when agents join squads.

use std::collections::HashMap;

use aof_core::coordination::{AgentIntroduction, CoordinationEvent};

use crate::types::{Agent, Soul};

/// Build an introduction event for a single agent
///
/// Given an Agent (from AGENTS.md) and optional Soul (from SOUL.md),
/// construct a CoordinationEvent with AgentIntroduction data.
///
/// If no Soul is provided, uses sensible fallback values:
/// - intro_message: "I'm [name], your [role]."
/// - personality_summary: empty string
///
/// # Arguments
/// * `agent` - Agent definition from AGENTS.md
/// * `soul` - Optional Soul guidance from SOUL.md
/// * `session_id` - Current daemon session ID
pub fn build_introduction_event(
    agent: &Agent,
    soul: Option<&Soul>,
    session_id: &str,
) -> CoordinationEvent {
    let intro_message = soul
        .map(|s| s.default_intro.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("I'm {}, your {}.", agent.name, agent.role));

    let personality_summary = soul
        .map(|s| s.personality_summary.clone())
        .unwrap_or_default();

    let introduction = AgentIntroduction {
        agent_id: agent.id.clone(),
        agent_name: agent.name.clone(),
        role: agent.role.clone(),
        avatar: agent.avatar.clone(),
        intro_message,
        personality_summary,
        skills: agent.skills.clone(),
    };

    CoordinationEvent::agent_introduction(session_id, introduction)
}

/// Build introduction events for a batch of agents
///
/// Calls `build_introduction_event` for each agent, looking up the
/// corresponding Soul from the provided map. Produces one event per
/// agent with no duplicates and no missed agents.
///
/// # Arguments
/// * `agents` - List of all agents from AGENTS.md
/// * `souls` - Map of agent_id -> Soul from SOUL.md
/// * `session_id` - Current daemon session ID
pub fn build_introduction_event_batch(
    agents: &[Agent],
    souls: &HashMap<String, Soul>,
    session_id: &str,
) -> Vec<CoordinationEvent> {
    agents
        .iter()
        .map(|agent| {
            let soul = souls.get(&agent.id);
            build_introduction_event(agent, soul, session_id)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_agent() -> Agent {
        Agent {
            id: "k8s-monitor".to_string(),
            name: "Kubernetes Monitor".to_string(),
            role: "Infrastructure Specialist".to_string(),
            avatar: "\u{1F916}".to_string(),
            personality_traits: vec!["methodical".to_string()],
            can: vec!["kubectl operations".to_string()],
            cannot: vec!["modify RBAC".to_string()],
            skills: vec![
                "kubectl".to_string(),
                "pod-debugging".to_string(),
                "log-analysis".to_string(),
            ],
        }
    }

    fn test_soul() -> Soul {
        Soul {
            id: "k8s-monitor".to_string(),
            communication_style: "formal-technical".to_string(),
            tone: "calm-professional".to_string(),
            values: vec!["system-stability".to_string()],
            personality_summary: "A methodical Kubernetes specialist who takes system health seriously.".to_string(),
            boundaries: vec!["Never trade stability for speed".to_string()],
            default_intro: "I'm Kubernetes Monitor, your infrastructure specialist. I watch your clusters constantly.".to_string(),
            communication_guide: "You are methodical and data-driven.".to_string(),
        }
    }

    #[test]
    fn test_build_introduction_event_with_soul() {
        let agent = test_agent();
        let soul = test_soul();

        let event = build_introduction_event(&agent, Some(&soul), "session-123");

        assert_eq!(event.agent_id, "k8s-monitor");
        assert_eq!(event.session_id, "session-123");
        assert!(event.introduction.is_some());

        let intro = event.introduction.unwrap();
        assert_eq!(intro.agent_name, "Kubernetes Monitor");
        assert_eq!(intro.role, "Infrastructure Specialist");
        assert_eq!(intro.avatar, "\u{1F916}");
        assert_eq!(
            intro.intro_message,
            "I'm Kubernetes Monitor, your infrastructure specialist. I watch your clusters constantly."
        );
        assert_eq!(
            intro.personality_summary,
            "A methodical Kubernetes specialist who takes system health seriously."
        );
        assert_eq!(intro.skills.len(), 3);
        assert_eq!(intro.skills[0], "kubectl");
    }

    #[test]
    fn test_build_introduction_event_without_soul() {
        let agent = test_agent();

        let event = build_introduction_event(&agent, None, "session-456");

        assert_eq!(event.agent_id, "k8s-monitor");
        let intro = event.introduction.unwrap();
        assert_eq!(
            intro.intro_message,
            "I'm Kubernetes Monitor, your Infrastructure Specialist."
        );
        assert_eq!(intro.personality_summary, "");
        assert_eq!(intro.skills, vec!["kubectl", "pod-debugging", "log-analysis"]);
    }

    #[test]
    fn test_build_introduction_event_empty_default_intro() {
        let agent = test_agent();
        let mut soul = test_soul();
        soul.default_intro = String::new(); // empty default_intro

        let event = build_introduction_event(&agent, Some(&soul), "session-789");

        let intro = event.introduction.unwrap();
        // Should use fallback when default_intro is empty
        assert_eq!(
            intro.intro_message,
            "I'm Kubernetes Monitor, your Infrastructure Specialist."
        );
    }

    #[test]
    fn test_build_introduction_event_batch() {
        let agents = vec![
            Agent::new("agent-1", "Agent One", "Role One", "\u{1F916}"),
            Agent::new("agent-2", "Agent Two", "Role Two", "\u{1F50D}"),
            Agent::new("agent-3", "Agent Three", "Role Three", "\u{1F6A8}"),
        ];

        let mut souls = HashMap::new();
        souls.insert(
            "agent-1".to_string(),
            Soul {
                id: "agent-1".to_string(),
                communication_style: "formal".to_string(),
                tone: "calm".to_string(),
                values: vec![],
                personality_summary: "First agent".to_string(),
                boundaries: vec![],
                default_intro: "Hello from Agent One".to_string(),
                communication_guide: String::new(),
            },
        );
        // agent-2 and agent-3 have no soul entries

        let events = build_introduction_event_batch(&agents, &souls, "session-batch");

        assert_eq!(events.len(), 3);

        // Agent 1 should use soul intro
        let intro1 = events[0].introduction.as_ref().unwrap();
        assert_eq!(intro1.agent_id, "agent-1");
        assert_eq!(intro1.intro_message, "Hello from Agent One");

        // Agent 2 should use fallback
        let intro2 = events[1].introduction.as_ref().unwrap();
        assert_eq!(intro2.agent_id, "agent-2");
        assert_eq!(intro2.intro_message, "I'm Agent Two, your Role Two.");

        // Agent 3 should use fallback
        let intro3 = events[2].introduction.as_ref().unwrap();
        assert_eq!(intro3.agent_id, "agent-3");
        assert_eq!(intro3.intro_message, "I'm Agent Three, your Role Three.");
    }

    #[test]
    fn test_introduction_event_serialization() {
        let agent = test_agent();
        let soul = test_soul();

        let event = build_introduction_event(&agent, Some(&soul), "session-ser");
        let json = serde_json::to_string(&event).unwrap();

        // Verify JSON shape
        assert!(json.contains("\"agent_name\":\"Kubernetes Monitor\""));
        assert!(json.contains("\"role\":\"Infrastructure Specialist\""));
        assert!(json.contains("\"skills\""));

        // Verify round-trip
        let deserialized: CoordinationEvent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.introduction.is_some());
        assert_eq!(
            deserialized.introduction.unwrap().agent_name,
            "Kubernetes Monitor"
        );
    }

    #[test]
    fn test_batch_no_duplicates() {
        let agents = vec![
            Agent::new("agent-1", "Agent One", "Role One", "\u{1F916}"),
            Agent::new("agent-2", "Agent Two", "Role Two", "\u{1F50D}"),
        ];

        let events = build_introduction_event_batch(&agents, &HashMap::new(), "session-x");

        // No duplicate agent_ids
        let ids: Vec<&str> = events
            .iter()
            .map(|e| e.agent_id.as_str())
            .collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);

        // All event_ids are unique
        assert_ne!(events[0].event_id, events[1].event_id);
    }

    #[test]
    fn test_introduction_avatar_preserved() {
        let mut agent = Agent::new("test", "Test", "Tester", "\u{1F916}");
        agent.skills = vec!["testing".to_string()];

        let event = build_introduction_event(&agent, None, "session-av");
        let intro = event.introduction.unwrap();
        assert_eq!(intro.avatar, "\u{1F916}");
    }

    #[test]
    fn test_introduction_with_empty_skills() {
        let agent = Agent::new("test", "Test", "Tester", "\u{1F916}");
        // Skills are empty by default from Agent::new

        let event = build_introduction_event(&agent, None, "session-es");
        let intro = event.introduction.unwrap();
        assert!(intro.skills.is_empty());
    }
}
