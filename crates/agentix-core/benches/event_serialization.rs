use criterion::{black_box, criterion_group, criterion_main, Criterion};
use agentix_core::{CoordinationEvent, ActivityEvent, ActivityType, AgentIntroduction};
use chrono::Utc;
use std::collections::HashMap;

fn create_sample_event() -> CoordinationEvent {
    CoordinationEvent {
        activity: ActivityEvent {
            activity_type: ActivityType::Thinking,
            message: "Analyzing pod logs for crash patterns...".to_string(),
            timestamp: Utc::now(),
            details: None,
        },
        agent_id: "test-agent".to_string(),
        session_id: "session-123".to_string(),
        event_id: "evt-456".to_string(),
        timestamp: Utc::now(),
        introduction: None,
        coordination_activity: None,
    }
}

fn create_session_state() -> HashMap<String, Vec<CoordinationEvent>> {
    let mut state = HashMap::new();
    let events: Vec<CoordinationEvent> = (0..10)
        .map(|i| {
            let mut event = create_sample_event();
            event.agent_id = format!("agent-{}", i);
            event.event_id = format!("evt-{}", i);
            event
        })
        .collect();
    state.insert("session-123".to_string(), events);
    state
}

fn create_event_with_introduction() -> CoordinationEvent {
    let mut event = create_sample_event();
    event.introduction = Some(AgentIntroduction {
        agent_id: "k8s-monitor".to_string(),
        agent_name: "Kubernetes Monitor".to_string(),
        role: "Infrastructure Specialist".to_string(),
        avatar: "🤖".to_string(),
        intro_message: "I monitor Kubernetes clusters and diagnose pod issues.".to_string(),
        personality_summary: "A methodical and detail-oriented infrastructure specialist.".to_string(),
        skills: vec!["kubectl".to_string(), "pod-debugging".to_string(), "log-analysis".to_string()],
    });
    event
}

fn bench_serialize_coordination_event(c: &mut Criterion) {
    let event = create_sample_event();

    c.bench_function("serialize_coordination_event", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&event)).unwrap();
            black_box(json);
        });
    });
}

fn bench_deserialize_coordination_event(c: &mut Criterion) {
    let event = create_sample_event();
    let json = serde_json::to_string(&event).unwrap();

    c.bench_function("deserialize_coordination_event", |b| {
        b.iter(|| {
            let event: CoordinationEvent = serde_json::from_str(black_box(&json)).unwrap();
            black_box(event);
        });
    });
}

fn bench_serialize_session_state(c: &mut Criterion) {
    let state = create_session_state();

    c.bench_function("serialize_session_state", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&state)).unwrap();
            black_box(json);
        });
    });
}

fn bench_clone_coordination_event(c: &mut Criterion) {
    let event = create_sample_event();

    c.bench_function("clone_coordination_event", |b| {
        b.iter(|| {
            let cloned = black_box(&event).clone();
            black_box(cloned);
        });
    });
}

fn bench_serialize_with_introduction(c: &mut Criterion) {
    let event = create_event_with_introduction();

    c.bench_function("serialize_event_with_introduction", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&event)).unwrap();
            black_box(json);
        });
    });
}

criterion_group! {
    name = event_serialization;
    config = Criterion::default().sample_size(100).significance_level(0.1);
    targets =
        bench_serialize_coordination_event,
        bench_deserialize_coordination_event,
        bench_serialize_session_state,
        bench_clone_coordination_event,
        bench_serialize_with_introduction
}

criterion_main!(event_serialization);
