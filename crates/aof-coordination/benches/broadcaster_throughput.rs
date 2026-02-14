use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aof_coordination::EventBroadcaster;
use aof_core::{CoordinationEvent, ActivityEvent, ActivityType};
use chrono::Utc;

fn create_sample_event(id: usize) -> CoordinationEvent {
    CoordinationEvent {
        activity: ActivityEvent {
            activity_type: ActivityType::Thinking,
            message: format!("Processing event {}", id),
            timestamp: Utc::now(),
            details: None,
        },
        agent_id: format!("agent-{}", id),
        session_id: "session-123".to_string(),
        event_id: format!("evt-{}", id),
        timestamp: Utc::now(),
        introduction: None,
        coordination_activity: None,
    }
}

fn bench_broadcast_single_subscriber(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("broadcast_1000_events_1_subscriber", |b| {
        b.iter(|| {
            rt.block_on(async {
                let broadcaster = EventBroadcaster::new(1000);
                let mut rx = broadcaster.subscribe();

                // Emit 1000 events
                for i in 0..1000 {
                    broadcaster.emit(create_sample_event(i));
                }

                // Receive all events
                let mut count = 0;
                while rx.try_recv().is_ok() {
                    count += 1;
                }
                black_box(count);
            });
        });
    });
}

fn bench_broadcast_50_subscribers(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("broadcast_1000_events_50_subscribers", |b| {
        b.iter(|| {
            rt.block_on(async {
                let broadcaster = EventBroadcaster::new(2000);

                // Create 50 subscribers
                let mut receivers = Vec::new();
                for _ in 0..50 {
                    receivers.push(broadcaster.subscribe());
                }

                // Emit 1000 events
                for i in 0..1000 {
                    broadcaster.emit(create_sample_event(i));
                }

                // Each subscriber receives events
                let mut total_received = 0;
                for rx in &mut receivers {
                    let mut count = 0;
                    while rx.try_recv().is_ok() {
                        count += 1;
                    }
                    total_received += count;
                }
                black_box(total_received);
            });
        });
    });
}

fn bench_subscriber_creation(c: &mut Criterion) {
    let broadcaster = EventBroadcaster::new(1000);

    c.bench_function("subscriber_creation_destruction", |b| {
        b.iter(|| {
            let rx = broadcaster.subscribe();
            black_box(rx);
            // rx is dropped here
        });
    });
}

fn bench_emit_no_subscribers(c: &mut Criterion) {
    let broadcaster = EventBroadcaster::new(1000);
    let event = create_sample_event(0);

    c.bench_function("emit_no_subscribers", |b| {
        b.iter(|| {
            broadcaster.emit(black_box(event.clone()));
        });
    });
}

fn bench_emit_with_10_subscribers(c: &mut Criterion) {
    let broadcaster = EventBroadcaster::new(1000);
    let _receivers: Vec<_> = (0..10).map(|_| broadcaster.subscribe()).collect();
    let event = create_sample_event(0);

    c.bench_function("emit_with_10_subscribers", |b| {
        b.iter(|| {
            broadcaster.emit(black_box(event.clone()));
        });
    });
}

criterion_group! {
    name = broadcaster_throughput;
    config = Criterion::default().sample_size(100).significance_level(0.1);
    targets =
        bench_broadcast_single_subscriber,
        bench_broadcast_50_subscribers,
        bench_subscriber_creation,
        bench_emit_no_subscribers,
        bench_emit_with_10_subscribers
}

criterion_main!(broadcaster_throughput);
