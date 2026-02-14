use criterion::{black_box, criterion_group, criterion_main, Criterion};
use aof_coordination_protocols::metrics::{TokenMetrics, DegradationConfig, DegradationManager};
use aof_coordination_protocols::heartbeat::{HeartbeatScheduler, HeartbeatConfig};
use std::sync::Arc;
use std::time::Duration;

fn bench_record_token_usage(c: &mut Criterion) {
    let metrics = TokenMetrics::new(Duration::from_secs(3600));

    c.bench_function("record_10000_token_events", |b| {
        b.iter(|| {
            for i in 0..10000 {
                if i % 2 == 0 {
                    metrics.record_coordination(black_box(50), black_box(30), "heartbeat");
                } else {
                    metrics.record_production(black_box(5000), black_box(3000));
                }
            }
        });
    });
}

fn bench_compute_overhead_percentage(c: &mut Criterion) {
    let metrics = TokenMetrics::new(Duration::from_secs(3600));

    // Populate with some data
    for _ in 0..1000 {
        metrics.record_coordination(50, 30, "heartbeat");
        metrics.record_production(5000, 3000);
    }

    c.bench_function("compute_overhead_percentage", |b| {
        b.iter(|| {
            let overhead = metrics.coordination_overhead();
            black_box(overhead);
        });
    });
}

fn bench_health_snapshot_10_agents(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("health_snapshot_10_agents", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = HeartbeatConfig::default();
                let (event_tx, _) = tokio::sync::broadcast::channel(1000);
                let scheduler = HeartbeatScheduler::new(config, event_tx, "session-123".to_string());

                // Register 10 agents
                for i in 0..10 {
                    let agent_id = format!("agent-{}", i);
                    scheduler.register_agent(&agent_id).await;
                }

                // Get health snapshot
                let snapshot = scheduler.agent_health_snapshot().await;
                black_box(snapshot);
            });
        });
    });
}

fn bench_degradation_evaluate(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("degradation_evaluate", |b| {
        b.iter(|| {
            rt.block_on(async {
                let metrics = Arc::new(TokenMetrics::new(Duration::from_secs(3600)));

                // Simulate high overhead
                for _ in 0..100 {
                    metrics.record_coordination(1000, 500, "heartbeat");
                    metrics.record_production(2000, 1000);
                }

                let config = DegradationConfig::default();
                let manager = DegradationManager::new(config, metrics);

                // Evaluate degradation
                let result = manager.evaluate().await;
                black_box(result);
            });
        });
    });
}

fn bench_metrics_snapshot(c: &mut Criterion) {
    let metrics = TokenMetrics::new(Duration::from_secs(3600));

    // Populate with data
    for _ in 0..1000 {
        metrics.record_coordination(50, 30, "heartbeat");
        metrics.record_coordination(100, 50, "standup");
        metrics.record_production(5000, 3000);
    }

    c.bench_function("metrics_snapshot", |b| {
        b.iter(|| {
            let overhead = metrics.coordination_overhead();
            let coord_tokens = metrics.total_coordination_tokens();
            let prod_tokens = metrics.total_production_tokens();
            let heartbeat = metrics.heartbeat_tokens();
            let standup = metrics.standup_tokens();
            black_box((overhead, coord_tokens, prod_tokens, heartbeat, standup));
        });
    });
}

criterion_group! {
    name = coordination_overhead;
    config = Criterion::default().sample_size(100).significance_level(0.1);
    targets =
        bench_record_token_usage,
        bench_compute_overhead_percentage,
        bench_health_snapshot_10_agents,
        bench_degradation_evaluate,
        bench_metrics_snapshot
}

criterion_main!(coordination_overhead);
