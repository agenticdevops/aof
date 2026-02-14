# Phase 8: Production Deployment Research

> Research document for production-ready deployment patterns for the AOF daemon

**Status**: Draft
**Last Updated**: 2026-02-14
**Focus**: What do I need to know to PLAN production-ready deployment?

---

## Executive Summary

AOF needs robust production deployment options that support:
- **Local-first architecture**: Daemon runs on user's machine by default
- **Optional server deployment** (INFR-05): Same daemon deployed to servers for always-on agents
- **Single binary distribution**: Cross-compiled binaries with curl | bash installation
- **Multiple deployment models**: systemd, Docker, Kubernetes, cloud platforms
- **Zero-downtime upgrades**: Graceful shutdown, state migration, backwards compatibility

**Key Finding**: AOF already has solid foundations (GitHub Actions release workflow, install script, Dockerfile, systemd example). Phase 8 should focus on:
1. Health check endpoints (`/health`, `/ready`, `/metrics`)
2. Structured logging with tracing
3. Prometheus metrics integration
4. Systemd hardening (already partially implemented)
5. Upgrade/rollback strategy with session state migration

---

## 1. Deployment Architecture

### 1.1. Deployment Models

```
┌─────────────────────────────────────────────────────────────┐
│                     Deployment Options                       │
└─────────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌──────────┐         ┌──────────┐         ┌──────────┐
    │  Local   │         │  Server  │         │  Cloud   │
    │  Desktop │         │ (systemd)│         │   (K8s)  │
    └──────────┘         └──────────┘         └──────────┘
         │                     │                     │
         │                     │                     │
         ▼                     ▼                     ▼
   Single binary       systemd service      StatefulSet
   + Web UI (opt)      + reverse proxy      + PVC + Redis
   + MCP servers       + log aggregation    + observability
```

**Deployment Model Decision Matrix:**

| Model | Use Case | Pros | Cons |
|-------|----------|------|------|
| **Local Binary** | Developer workflows, personal use | Zero setup, no infrastructure | No HA, manual updates |
| **systemd Service** | Single-server deployments | Simple, auto-restart, journald logs | Single point of failure |
| **Docker** | Containerized environments | Portable, versioned, resource limits | Requires Docker runtime |
| **Kubernetes** | Large-scale, multi-tenant | HA, auto-scaling, observability | Complex setup |

### 1.2. Architecture Diagram

```
┌───────────────────────────────────────────────────────────────────┐
│                         Internet                                   │
│  (LLM providers, MCP servers, messaging platforms)                │
└───────────────────────────────────────────────────────────────────┘
                             │
                             │ HTTPS/443
                             ▼
┌───────────────────────────────────────────────────────────────────┐
│                    Reverse Proxy (nginx)                          │
│  - HTTPS termination                                               │
│  - WebSocket upgrade                                               │
│  - Rate limiting                                                   │
│  - Static asset caching                                            │
└───────────────────────────────────────────────────────────────────┘
                             │
                             │ HTTP/8080
                             ▼
┌───────────────────────────────────────────────────────────────────┐
│                    AOF Daemon (aofctl serve)                      │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  Axum HTTP Server                                        │     │
│  ├─────────────────────────────────────────────────────────┤     │
│  │  /health              - Health check (liveness)          │     │
│  │  /ready               - Readiness check (dependencies)   │     │
│  │  /metrics             - Prometheus metrics               │     │
│  │  /api/*               - REST API (config, status)        │     │
│  │  /ws                  - WebSocket (events, logs)         │     │
│  │  /webhook/:platform   - Trigger webhooks                 │     │
│  │  /*                   - Static files (Mission Control)   │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                    │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  Core Services                                           │     │
│  │  - EventBroadcaster (WebSocket events)                  │     │
│  │  - TriggerHandler (platform adapters)                   │     │
│  │  - RuntimeOrchestrator (agent execution)                │     │
│  │  - CoordinationManager (fleet coordination)             │     │
│  │  - SessionPersistence (state management)                │     │
│  └─────────────────────────────────────────────────────────┘     │
└───────────────────────────────────────────────────────────────────┘
                             │
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│ Filesystem  │      │    Redis    │      │   Syslog    │
│ - Agents    │      │  (optional) │      │ (optional)  │
│ - Tools     │      │  - Memory   │      │  - Logs     │
│ - Sessions  │      │  - Sessions │      │  - Metrics  │
│ - Decisions │      │  - Locks    │      │  - Events   │
└─────────────┘      └─────────────┘      └─────────────┘
```

---

## 2. Binary Distribution & Installation

### 2.1. Current Implementation (GitHub Actions)

**Strengths:**
- ✅ Automated multi-platform builds (Linux x86_64/aarch64, macOS Intel/Apple Silicon, Windows)
- ✅ SHA256 checksums for integrity verification
- ✅ GitHub Releases with formatted release notes
- ✅ Installation script with platform detection

**Workflow:** `.github/workflows/release.yml`
```yaml
# Triggers on version tags (v*)
# Builds for 6 platforms (Linux/macOS/Windows × x86_64/aarch64)
# Generates checksums
# Creates GitHub Release
# Publishes crates to crates.io
# Deploys install.sh to docs.aof.sh
```

**Installation Script:** `scripts/install.sh`
```bash
# Features:
- Auto-detects OS and CPU architecture
- Downloads from GitHub releases
- Verifies SHA256 checksums
- Installs to ~/.local/bin by default
- Warns if binary not in PATH
- Supports --version, --install-dir, --verbose flags
```

### 2.2. Version Management

**Current:** Semantic versioning (v0.4.0-beta)

**Recommendation:**
```toml
# Cargo.toml
[workspace.package]
version = "1.0.0"  # MAJOR.MINOR.PATCH

# Version embedding
# Add to aofctl/src/cli.rs:
const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_COMMIT: &str = env!("VERGEN_GIT_SHA");
const BUILD_DATE: &str = env!("VERGEN_BUILD_TIMESTAMP");

// aofctl --version output:
// aofctl 1.0.0 (commit: abc123, built: 2024-01-15)
```

**Auto-Update Strategy:**
```bash
# Check for updates (opt-in)
aofctl update --check
# New version available: v1.1.0 (current: v1.0.0)
# Run: aofctl update to upgrade

aofctl update
# Downloading aofctl-v1.1.0...
# Backup: ~/.local/bin/aofctl.backup
# Install: ~/.local/bin/aofctl
# Upgrade complete. Run: aofctl --version
```

### 2.3. Binary Size Optimization

**Current:** ~50MB release binary (from existing docs)

**Optimization Checklist:**
```toml
# Cargo.toml (already implemented)
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true

# Additional:
[profile.release]
panic = "abort"           # Remove panic unwinding code
opt-level = "z"           # Optimize for size (vs. 3 for speed)
```

**UPX Compression (optional):**
```bash
# Compress binary (can reduce by 50%)
upx --best --lzma target/release/aofctl
# Trade-off: slower startup time
```

---

## 3. Systemd Service Deployment

### 3.1. Service Unit Template

**File:** `/etc/systemd/system/aof-daemon.service`

```ini
[Unit]
Description=AOF Daemon - Agentic Ops Framework
Documentation=https://docs.aof.sh
After=network-online.target
Wants=network-online.target

# Dependencies (optional)
Requires=redis.service  # If using Redis memory backend
After=redis.service

[Service]
Type=simple
User=aof
Group=aof
WorkingDirectory=/opt/aof

# Environment
Environment="RUST_LOG=info,aofctl=info,aof_runtime=info"
Environment="AOF_CONFIG_DIR=/etc/aof"
Environment="AOF_DATA_DIR=/var/lib/aof"
EnvironmentFile=-/etc/aof/daemon.env  # Optional secrets

# Execution
ExecStart=/usr/local/bin/aofctl serve \
    --config /etc/aof/daemon.yaml \
    --port 8080 \
    --host 0.0.0.0

# Restart policy
Restart=on-failure
RestartSec=5s
StartLimitInterval=300s
StartLimitBurst=5

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096
MemoryMax=2G
TasksMax=4096

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=aof-daemon

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectKernelLogs=true
ProtectControlGroups=true
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
LockPersonality=true
PrivateDevices=false  # Allow access to /dev/random for RNG

# File system access
ReadWritePaths=/var/lib/aof /var/log/aof
ReadOnlyPaths=/etc/aof

# Graceful shutdown
TimeoutStopSec=30s
KillMode=mixed
KillSignal=SIGTERM

[Install]
WantedBy=multi-user.target
```

### 3.2. Socket Activation (Advanced)

**Use Case:** Start daemon on first request (saves resources)

```ini
# /etc/systemd/system/aof-daemon.socket
[Unit]
Description=AOF Daemon Socket
PartOf=aof-daemon.service

[Socket]
ListenStream=8080
Accept=false

[Install]
WantedBy=sockets.target
```

```ini
# /etc/systemd/system/aof-daemon.service
[Service]
ExecStart=/usr/local/bin/aofctl serve --systemd-socket
# Daemon binds to socket passed by systemd (fd 3)
```

### 3.3. Installation Script

**File:** `scripts/install-systemd.sh`

```bash
#!/bin/bash
set -e

# Create user
sudo useradd -r -s /bin/false aof || true

# Create directories
sudo mkdir -p /opt/aof /etc/aof /var/lib/aof /var/log/aof
sudo chown aof:aof /var/lib/aof /var/log/aof

# Copy binary
sudo cp target/release/aofctl /usr/local/bin/
sudo chmod +x /usr/local/bin/aofctl

# Copy config
sudo cp examples/daemon.yaml /etc/aof/
sudo chown aof:aof /etc/aof/daemon.yaml

# Install systemd service
sudo cp scripts/aof-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable aof-daemon
sudo systemctl start aof-daemon

# Verify
sudo systemctl status aof-daemon
echo "✅ AOF daemon installed and running"
echo "📋 Logs: sudo journalctl -u aof-daemon -f"
echo "🛠️ Config: /etc/aof/daemon.yaml"
```

### 3.4. Journald Integration

**Structured Logging:**
```rust
// Use tracing-journald for structured logging
use tracing_journald::layer;
use tracing_subscriber::prelude::*;

let journald = layer()?.with_syslog_identifier("aof-daemon".to_string());
let subscriber = tracing_subscriber::registry()
    .with(journald)
    .with(tracing_subscriber::fmt::layer());

tracing::subscriber::set_global_default(subscriber)?;

// Log with fields:
tracing::info!(
    agent_id = "agent-123",
    execution_id = "exec-456",
    duration_ms = 1234,
    "Agent execution completed"
);
```

**Query Logs:**
```bash
# Follow logs
sudo journalctl -u aof-daemon -f

# Filter by priority
sudo journalctl -u aof-daemon -p err

# Filter by field
sudo journalctl -u aof-daemon AGENT_ID=agent-123

# Export JSON
sudo journalctl -u aof-daemon -o json-pretty
```

---

## 4. Docker Deployment

### 4.1. Multi-Stage Dockerfile (Current)

**File:** `Dockerfile` (already exists)

**Analysis:**
- ✅ Multi-stage build (builder + runtime)
- ✅ Dependency caching via placeholder files
- ✅ Non-root user (aof:1000)
- ✅ Health check configured
- ✅ Minimal runtime image (Debian slim)

**Improvements:**
```dockerfile
# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \  # For health checks
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -u 1000 -s /bin/false aof

# Create directories
RUN mkdir -p /app/agents /app/config /app/checkpoints /app/data
RUN chown -R aof:aof /app

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/aofctl /usr/local/bin/aofctl

# Copy examples
COPY --from=builder /app/examples/ /app/examples/

# Switch to non-root
USER aof

# Expose ports
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Environment defaults
ENV RUST_LOG=info,aofctl=info,aof_runtime=info
ENV AOF_CONFIG_DIR=/app/config
ENV AOF_DATA_DIR=/app/data

# Entrypoint
ENTRYPOINT ["/usr/local/bin/aofctl"]
CMD ["serve", "--config", "/app/config/daemon.yaml", "--port", "8080"]
```

### 4.2. Docker Compose Stack

**File:** `docker-compose.yaml` (production-ready)

```yaml
version: '3.8'

services:
  # AOF Daemon
  aof-daemon:
    build:
      context: .
      dockerfile: Dockerfile
    image: aof:latest
    container_name: aof-daemon
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./config:/app/config:ro
      - aof-data:/app/data
      - aof-checkpoints:/app/checkpoints
    environment:
      - RUST_LOG=info,aofctl=info,aof_runtime=info
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - AOF_MEMORY_BACKEND=redis
      - REDIS_URL=redis://redis:6379
    depends_on:
      redis:
        condition: service_healthy
    networks:
      - aof-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

  # Redis (optional, for memory backend)
  redis:
    image: redis:7-alpine
    container_name: aof-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data
    networks:
      - aof-network
    command: redis-server --appendonly yes --maxmemory 512mb --maxmemory-policy allkeys-lru
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 3

  # Prometheus (monitoring)
  prometheus:
    image: prom/prometheus:latest
    container_name: aof-prometheus
    restart: unless-stopped
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.time=30d'
    networks:
      - aof-network

  # Grafana (dashboards)
  grafana:
    image: grafana/grafana:latest
    container_name: aof-grafana
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin}
      - GF_INSTALL_PLUGINS=redis-datasource
    depends_on:
      - prometheus
    networks:
      - aof-network

volumes:
  aof-data:
  aof-checkpoints:
  redis-data:
  prometheus-data:
  grafana-data:

networks:
  aof-network:
    driver: bridge
```

### 4.3. Health Check Endpoints

**Implementation in `serve.rs`:**

```rust
// Health check (liveness probe)
async fn health_handler() -> impl IntoResponse {
    AxumJson(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": get_uptime()
    }))
}

// Readiness check (checks dependencies)
async fn ready_handler(
    State(state): State<AppState>
) -> Result<impl IntoResponse, StatusCode> {
    // Check Redis connection
    if let Some(redis) = &state.redis {
        if redis.ping().await.is_err() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    // Check disk space
    if available_disk_space("/app/data")? < 100_000_000 {  // 100MB
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(AxumJson(serde_json::json!({
        "status": "ready",
        "dependencies": {
            "redis": "ok",
            "disk_space": "ok"
        }
    })))
}

// Add to router:
Router::new()
    .route("/health", get(health_handler))
    .route("/ready", get(ready_handler))
    .route("/metrics", get(metrics_handler))  // Prometheus
```

### 4.4. Volume Management

**Production Volumes:**
```yaml
volumes:
  # Persistent data (agents, workflows, sessions)
  aof-data:
    driver: local
    driver_opts:
      type: none
      device: /mnt/aof-data
      o: bind

  # Session checkpoints (fast SSD recommended)
  aof-checkpoints:
    driver: local
    driver_opts:
      type: none
      device: /mnt/nvme/aof-checkpoints
      o: bind
```

**Backup Strategy:**
```bash
# Backup volumes
docker run --rm \
  -v aof-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar czf /backup/aof-data-$(date +%Y%m%d).tar.gz /data

# Restore volumes
docker run --rm \
  -v aof-data:/data \
  -v $(pwd)/backups:/backup \
  alpine tar xzf /backup/aof-data-20240115.tar.gz -C /
```

---

## 5. Kubernetes Deployment

### 5.1. StatefulSet vs Deployment

**Decision Matrix:**

| Aspect | Deployment | StatefulSet |
|--------|------------|-------------|
| Use Case | Stateless agents | Stateful daemon |
| Pod Identity | Random | Stable (aof-0, aof-1) |
| Storage | Ephemeral or shared PVC | Per-pod PVC |
| Network | ClusterIP | Headless Service |
| Scaling | Fast | Ordered |
| **Recommendation** | Agent execution | Daemon server |

**For AOF:** Use **StatefulSet** if running daemon mode with session persistence. Use **Deployment** if agents are ephemeral.

### 5.2. StatefulSet Manifest

**File:** `k8s/statefulset.yaml`

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: aof-daemon
  namespace: aof-system
  labels:
    app: aof-daemon
spec:
  serviceName: aof-daemon-headless
  replicas: 3
  selector:
    matchLabels:
      app: aof-daemon

  # Update strategy
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 0  # Update all pods

  # Pod template
  template:
    metadata:
      labels:
        app: aof-daemon
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8080"
        prometheus.io/path: "/metrics"

    spec:
      serviceAccountName: aof-daemon
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000

      # Init containers
      initContainers:
      - name: wait-for-redis
        image: busybox:latest
        command: ['sh', '-c', 'until nc -z aof-redis 6379; do sleep 1; done']

      containers:
      - name: daemon
        image: aof:latest
        imagePullPolicy: Always

        ports:
        - containerPort: 8080
          name: http
          protocol: TCP

        env:
        - name: RUST_LOG
          value: "info,aofctl=info,aof_runtime=info"
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: aof-secrets
              key: anthropic-api-key
        - name: AOF_MEMORY_BACKEND
          value: "redis"
        - name: REDIS_URL
          value: "redis://aof-redis:6379"
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace

        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: data
          mountPath: /app/data
        - name: checkpoints
          mountPath: /app/checkpoints

        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"

        # Liveness probe (restart if unhealthy)
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3

        # Readiness probe (remove from load balancer if not ready)
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3

        # Lifecycle hooks
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 10"]  # Graceful shutdown

      volumes:
      - name: config
        configMap:
          name: aof-config

  # Volume claim templates (per-pod storage)
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: ssd
      resources:
        requests:
          storage: 10Gi

  - metadata:
      name: checkpoints
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 5Gi
```

### 5.3. Service Manifests

**Headless Service (StatefulSet DNS):**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: aof-daemon-headless
  namespace: aof-system
spec:
  clusterIP: None
  selector:
    app: aof-daemon
  ports:
  - port: 8080
    targetPort: 8080
    name: http
```

**ClusterIP Service (Load balanced):**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: aof-daemon
  namespace: aof-system
spec:
  type: ClusterIP
  selector:
    app: aof-daemon
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
    name: http
```

### 5.4. Leader Election (HA Setup)

**Use Case:** Only one daemon should perform periodic tasks (cleanup, cron jobs)

**Implementation:**
```rust
use kube::runtime::controller::Action;
use kube_runtime::watcher::Config as WatcherConfig;
use k8s_openapi::api::coordination::v1::Lease;

// Leader election logic
async fn run_with_leader_election() -> Result<()> {
    let client = kube::Client::try_default().await?;
    let leases: Api<Lease> = Api::namespaced(client, "aof-system");

    let lease_name = "aof-daemon-leader";
    let pod_name = std::env::var("POD_NAME")?;

    loop {
        // Try to acquire lease
        if try_acquire_lease(&leases, lease_name, &pod_name).await? {
            info!("Became leader");
            run_leader_tasks().await?;
        } else {
            info!("Not leader, sleeping");
            sleep(Duration::from_secs(30)).await;
        }
    }
}
```

### 5.5. Persistent Volume Claims

**StorageClass (SSD):**
```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/gce-pd  # Or AWS EBS, Azure Disk
parameters:
  type: pd-ssd
  replication-type: regional-pd
allowVolumeExpansion: true
```

---

## 6. Observability in Production

### 6.1. Structured Logging

**Implementation:**
```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use serde_json::json;

// JSON logging for production
let format = fmt::format()
    .json()
    .flatten_event(true)
    .with_current_span(false);

tracing_subscriber::registry()
    .with(EnvFilter::from_default_env())
    .with(fmt::layer().event_format(format))
    .init();

// Usage:
tracing::info!(
    agent_id = "agent-123",
    execution_id = "exec-456",
    duration_ms = 1234,
    tokens_used = 5000,
    "Agent execution completed"
);

// Output (JSON):
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "level": "INFO",
  "message": "Agent execution completed",
  "agent_id": "agent-123",
  "execution_id": "exec-456",
  "duration_ms": 1234,
  "tokens_used": 5000,
  "target": "aof_runtime::agent"
}
```

**Log Aggregation (Loki):**
```yaml
# Promtail config (ships logs to Loki)
scrape_configs:
- job_name: aof-daemon
  kubernetes_sd_configs:
  - role: pod
    namespaces:
      names:
      - aof-system
  relabel_configs:
  - source_labels: [__meta_kubernetes_pod_label_app]
    regex: aof-daemon
    action: keep
  pipeline_stages:
  - json:
      expressions:
        level: level
        agent_id: agent_id
        execution_id: execution_id
  - labels:
      level:
      agent_id:
      execution_id:
```

### 6.2. Prometheus Metrics

**Metrics to Track:**

| Metric | Type | Description |
|--------|------|-------------|
| `aof_agent_executions_total` | Counter | Total agent executions |
| `aof_agent_execution_duration_seconds` | Histogram | Execution duration |
| `aof_agent_execution_errors_total` | Counter | Execution errors |
| `aof_llm_requests_total` | Counter | LLM API calls |
| `aof_llm_tokens_total` | Counter | Tokens consumed |
| `aof_llm_latency_seconds` | Histogram | LLM API latency |
| `aof_webhook_requests_total` | Counter | Webhook requests |
| `aof_memory_operations_total` | Counter | Memory ops |
| `aof_active_sessions` | Gauge | Active sessions |
| `aof_queue_depth` | Gauge | Pending tasks |

**Implementation:**
```rust
use prometheus::{
    Counter, Histogram, HistogramOpts, Registry, Encoder, TextEncoder,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref AGENT_EXECUTIONS: Counter = Counter::new(
        "aof_agent_executions_total",
        "Total agent executions"
    ).unwrap();

    pub static ref EXECUTION_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "aof_agent_execution_duration_seconds",
            "Agent execution duration"
        ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0])
    ).unwrap();

    pub static ref LLM_TOKENS: Counter = Counter::new(
        "aof_llm_tokens_total",
        "Total LLM tokens consumed"
    ).unwrap();
}

// Register metrics
pub fn init_metrics() -> Result<()> {
    REGISTRY.register(Box::new(AGENT_EXECUTIONS.clone()))?;
    REGISTRY.register(Box::new(EXECUTION_DURATION.clone()))?;
    REGISTRY.register(Box::new(LLM_TOKENS.clone()))?;
    Ok(())
}

// Metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .header("Content-Type", encoder.format_type())
        .body(Body::from(buffer))
        .unwrap()
}

// Usage in code
AGENT_EXECUTIONS.inc();
EXECUTION_DURATION.observe(duration.as_secs_f64());
LLM_TOKENS.inc_by(response.usage.total_tokens as u64);
```

**Prometheus Scrape Config:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'aof-daemon'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names: [aof-system]
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        target_label: __address__
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
```

### 6.3. Health Check Endpoints

**Implementation Status:** Partially exists in `serve.rs`

**Required Endpoints:**

| Endpoint | Purpose | Response |
|----------|---------|----------|
| `GET /health` | Liveness probe | `{"status": "ok", "version": "1.0.0"}` |
| `GET /ready` | Readiness probe | `{"status": "ready", "dependencies": {...}}` |
| `GET /metrics` | Prometheus metrics | Text format metrics |

**Enhanced Health Check:**
```rust
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_seconds: u64,
    git_commit: String,
    build_date: String,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: String,
    dependencies: DependencyStatus,
}

#[derive(Serialize)]
struct DependencyStatus {
    redis: String,         // "ok" | "unavailable"
    disk_space: String,    // "ok" | "low" | "critical"
    llm_provider: String,  // "ok" | "unreachable"
}

async fn ready_handler(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let mut deps = DependencyStatus {
        redis: "unavailable".to_string(),
        disk_space: "unknown".to_string(),
        llm_provider: "unknown".to_string(),
    };

    // Check Redis
    if let Some(redis) = &state.redis {
        deps.redis = if redis.ping().await.is_ok() { "ok" } else { "unavailable" };
    }

    // Check disk space
    let available = available_disk_space(&state.data_dir)?;
    deps.disk_space = if available > 1_000_000_000 {
        "ok"
    } else if available > 100_000_000 {
        "low"
    } else {
        "critical"
    };

    // Check LLM provider (simple HEAD request)
    let client = reqwest::Client::new();
    deps.llm_provider = if client.head("https://api.anthropic.com").send().await.is_ok() {
        "ok"
    } else {
        "unreachable"
    };

    // Fail readiness if critical dependencies unavailable
    if deps.disk_space == "critical" {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(AxumJson(ReadinessResponse {
        status: "ready".to_string(),
        dependencies: deps,
    }))
}
```

### 6.4. Debugging Endpoints (Development Only)

**Debug-only endpoints (disabled in production):**

```rust
// Only in debug builds
#[cfg(debug_assertions)]
{
    router = router
        .route("/debug/config", get(debug_config))
        .route("/debug/state", get(debug_state))
        .route("/debug/sessions", get(debug_sessions));
}
```

---

## 7. Upgrade & Rollback Strategy

### 7.1. Backwards Compatibility Guarantees

**Compatibility Matrix:**

| Version | Config Format | API | State Format |
|---------|---------------|-----|--------------|
| 1.0.0 → 1.0.x | ✅ Compatible | ✅ Compatible | ✅ Compatible |
| 1.0.0 → 1.1.0 | ⚠️ Deprecated fields warned | ✅ Compatible | ✅ Compatible |
| 1.0.0 → 2.0.0 | ❌ Breaking changes | ❌ Breaking changes | ⚠️ Migration required |

**Semantic Versioning Contract:**
- **PATCH (1.0.0 → 1.0.1)**: Bug fixes, no breaking changes, drop-in replacement
- **MINOR (1.0.0 → 1.1.0)**: New features, deprecations warned, backwards compatible
- **MAJOR (1.0.0 → 2.0.0)**: Breaking changes, migration guide provided

### 7.2. State Migration Strategy

**Session State Format Versioning:**
```rust
#[derive(Serialize, Deserialize)]
struct SessionState {
    version: u32,  // Schema version
    data: SessionData,
}

// Migration on load
fn load_session(path: &Path) -> Result<SessionState> {
    let raw: SessionState = serde_json::from_str(&fs::read_to_string(path)?)?;

    match raw.version {
        1 => Ok(raw),  // Current version
        0 => migrate_v0_to_v1(raw),  // Migrate from v0
        _ => Err(anyhow!("Unsupported session version: {}", raw.version)),
    }
}

fn migrate_v0_to_v1(old: SessionState) -> Result<SessionState> {
    // Migration logic
    Ok(SessionState {
        version: 1,
        data: /* migrated data */,
    })
}
```

**Database Schema Migrations (Redis):**
```rust
// Migration on startup
async fn run_migrations(redis: &Redis) -> Result<()> {
    let current_version: u32 = redis.get("schema_version").await.unwrap_or(0);

    if current_version < 1 {
        migrate_to_v1(redis).await?;
        redis.set("schema_version", 1).await?;
    }

    if current_version < 2 {
        migrate_to_v2(redis).await?;
        redis.set("schema_version", 2).await?;
    }

    Ok(())
}
```

### 7.3. Graceful Shutdown

**Implementation:**
```rust
use tokio::signal;
use tokio::sync::broadcast;

async fn serve() -> Result<()> {
    // Shutdown signal
    let (shutdown_tx, _) = broadcast::channel(1);

    // Spawn server
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            let _ = signal::ctrl_c().await;
            info!("Shutdown signal received, starting graceful shutdown");
        });

    // Wait for shutdown
    tokio::select! {
        _ = server => {},
        _ = signal::ctrl_c() => {
            info!("SIGINT received, shutting down");
        }
    }

    // Cleanup
    info!("Saving session state...");
    state.session_persistence.save_all().await?;

    info!("Closing database connections...");
    state.redis.close().await?;

    info!("Shutdown complete");
    Ok(())
}
```

**Systemd Integration:**
```ini
[Service]
# Graceful shutdown
TimeoutStopSec=30s
KillMode=mixed
KillSignal=SIGTERM

# Allow 30 seconds for cleanup
# Then SIGKILL if still running
```

### 7.4. Zero-Downtime Upgrade (Kubernetes)

**Rolling Update Strategy:**
```yaml
spec:
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 0           # Update all pods
      maxUnavailable: 1      # Keep 2/3 pods running during update
```

**Blue-Green Deployment:**
```bash
# Deploy new version (blue)
kubectl apply -f k8s/statefulset-v2.yaml

# Wait for new pods to be ready
kubectl rollout status statefulset/aof-daemon-v2

# Switch traffic
kubectl patch service aof-daemon -p '{"spec":{"selector":{"version":"v2"}}}'

# Verify
kubectl get pods -l app=aof-daemon,version=v2

# Rollback if needed
kubectl patch service aof-daemon -p '{"spec":{"selector":{"version":"v1"}}}'
kubectl delete statefulset aof-daemon-v2
```

### 7.5. Upgrade Procedure Checklist

**Pre-Upgrade:**
```bash
# 1. Backup current state
kubectl exec aof-daemon-0 -- tar czf /tmp/backup.tar.gz /app/data
kubectl cp aof-daemon-0:/tmp/backup.tar.gz ./backup-$(date +%Y%m%d).tar.gz

# 2. Verify health
kubectl exec aof-daemon-0 -- curl http://localhost:8080/health

# 3. Check resource usage
kubectl top pod -l app=aof-daemon

# 4. Review changelog
cat CHANGELOG.md
```

**Upgrade:**
```bash
# 5. Update image
kubectl set image statefulset/aof-daemon daemon=aof:v1.1.0

# 6. Monitor rollout
kubectl rollout status statefulset/aof-daemon

# 7. Verify new pods
kubectl get pods -l app=aof-daemon
kubectl logs aof-daemon-0 --tail=100
```

**Post-Upgrade:**
```bash
# 8. Run smoke tests
kubectl exec aof-daemon-0 -- aofctl run --config /app/examples/hello.yaml

# 9. Check metrics
curl http://aof-daemon.aof-system.svc.cluster.local/metrics | grep aof_agent_executions_total

# 10. Monitor for 1 hour
kubectl logs -f aof-daemon-0
```

**Rollback:**
```bash
# If issues occur
kubectl rollout undo statefulset/aof-daemon
kubectl rollout status statefulset/aof-daemon
```

---

## 8. Release Procedure

### 8.1. Automated Release Workflow

**Current:** `.github/workflows/release.yml` triggers on `v*` tags

**Release Steps:**
```bash
# 1. Update version
vim Cargo.toml  # Update [workspace.package] version

# 2. Update changelog
vim CHANGELOG.md

# 3. Commit
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to v1.1.0"

# 4. Create tag
git tag -a v1.1.0 -m "Release v1.1.0: Add health checks and metrics"

# 5. Push (triggers CI)
git push origin main
git push origin v1.1.0

# 6. Monitor CI
# https://github.com/agenticdevops/aof/actions

# 7. Verify release
# https://github.com/agenticdevops/aof/releases/tag/v1.1.0

# 8. Test installation
curl -sSL https://docs.aof.sh/install.sh | bash -s -- --version v1.1.0
```

### 8.2. Release Notes Template

**Auto-generated by workflow:**
```markdown
# AOF v1.1.0 Release

## Installation

### Using install script (recommended)
```bash
curl -sSL https://docs.aof.sh/install.sh | bash
```

### Manual download
Download the appropriate binary for your platform below and add it to your PATH.

**Supported platforms:**
- Linux x86_64
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
- Windows x86_64

### Verify checksum
```bash
sha256sum -c aofctl-*.sha256
```

## Changelog

### Added
- Health check endpoints (`/health`, `/ready`)
- Prometheus metrics (`/metrics`)
- Structured JSON logging
- Graceful shutdown handling

### Changed
- Improved Docker image size (50MB → 35MB)
- Updated Rust to 1.76

### Fixed
- Session state race condition
- Memory leak in WebSocket handler

## Breaking Changes
None (minor version bump)

## Migration Guide
No migration required for v1.0.x → v1.1.0.

## Getting Started

After installation, verify it works:
```bash
aofctl --version
aofctl api-resources
```

Check out the [documentation](https://docs.aof.sh) to get started!
```

---

## 9. Observability Checklist

### Production Readiness Checklist

**Deployment:**
- [ ] Health check endpoints implemented (`/health`, `/ready`)
- [ ] Prometheus metrics endpoint (`/metrics`)
- [ ] Structured logging (JSON format)
- [ ] Graceful shutdown handling
- [ ] Resource limits configured (CPU, memory)
- [ ] Security hardening (systemd, container)

**Monitoring:**
- [ ] Prometheus scraping configured
- [ ] Grafana dashboards created
- [ ] Alerting rules defined
- [ ] Log aggregation setup (Loki/ELK)
- [ ] Error tracking enabled

**Reliability:**
- [ ] Health probes configured (liveness, readiness)
- [ ] Auto-restart enabled (systemd/Kubernetes)
- [ ] Backup strategy defined
- [ ] Disaster recovery tested
- [ ] Rollback procedure documented

**Performance:**
- [ ] Resource usage profiled
- [ ] Load testing completed
- [ ] Database connection pooling
- [ ] Cache strategy implemented
- [ ] Rate limiting configured

**Security:**
- [ ] Secrets stored securely (Vault/K8s Secrets)
- [ ] TLS configured (reverse proxy)
- [ ] Network policies applied
- [ ] Security scanning (Trivy/Snyk)
- [ ] API authentication enabled

---

## 10. Next Steps for Implementation

### High Priority (Phase 8)

1. **Health Check Endpoints**
   - Implement `/health` (liveness)
   - Implement `/ready` (readiness with dependency checks)
   - Add to `serve.rs`

2. **Prometheus Metrics**
   - Define metrics registry
   - Implement `/metrics` endpoint
   - Add instrumentation to agent execution, LLM calls

3. **Structured Logging**
   - Migrate to `tracing-subscriber` with JSON format
   - Add structured fields (agent_id, execution_id, duration)
   - Configure log levels via RUST_LOG

4. **Graceful Shutdown**
   - Implement SIGTERM handler
   - Save session state on shutdown
   - Close database connections cleanly

5. **Documentation**
   - Update deployment guide with health checks
   - Add Prometheus/Grafana setup guide
   - Document upgrade procedure

### Medium Priority (Post-Phase 8)

6. **Kubernetes Manifests**
   - Create StatefulSet YAML
   - Add ServiceMonitor for Prometheus Operator
   - Create PodDisruptionBudget

7. **Monitoring Dashboards**
   - Create Grafana dashboard JSON
   - Define Prometheus alerting rules
   - Setup PagerDuty/Slack integration

8. **Load Testing**
   - Create k6 load test scripts
   - Benchmark agent execution throughput
   - Profile memory usage under load

### Low Priority (Future)

9. **Auto-Update**
   - Implement `aofctl update` command
   - Check for new versions
   - In-place binary replacement

10. **Advanced Deployment**
    - Leader election for HA
    - Multi-region deployment
    - Blue-green deployment automation

---

## 11. Reference Architecture Diagrams

### 11.1. Systemd Deployment

```
┌───────────────────────────────────────────────────────────┐
│                      Server (Ubuntu 22.04)                 │
│                                                            │
│  ┌──────────────────────────────────────────────────┐    │
│  │  systemd (aof-daemon.service)                     │    │
│  │  - Auto-restart on crash                          │    │
│  │  - Resource limits (CPU, memory)                  │    │
│  │  - Security hardening (NoNewPrivileges, etc.)     │    │
│  └──────────────────────────────────────────────────┘    │
│                        │                                   │
│                        ▼                                   │
│  ┌──────────────────────────────────────────────────┐    │
│  │  aofctl serve (port 8080)                         │    │
│  │  - HTTP/WebSocket server                          │    │
│  │  - Health checks (/health, /ready)                │    │
│  │  - Metrics endpoint (/metrics)                    │    │
│  └──────────────────────────────────────────────────┘    │
│                        │                                   │
│         ┌──────────────┼──────────────┐                   │
│         │              │              │                   │
│         ▼              ▼              ▼                   │
│  ┌───────────┐  ┌──────────┐  ┌──────────┐              │
│  │   Redis   │  │Filesystem│  │ journald │              │
│  │(optional) │  │ (state)  │  │  (logs)  │              │
│  └───────────┘  └──────────┘  └──────────┘              │
│                                                            │
└───────────────────────────────────────────────────────────┘
                        │
                        │ HTTPS
                        ▼
               ┌────────────────┐
               │ nginx (reverse │
               │     proxy)     │
               │ - TLS term     │
               │ - Rate limit   │
               └────────────────┘
                        │
                        ▼
                   Internet
```

### 11.2. Kubernetes Deployment

```
┌─────────────────────────────────────────────────────────────────┐
│                      Kubernetes Cluster                          │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  Namespace: aof-system                                  │    │
│  │                                                          │    │
│  │  ┌────────────────────────────────────────────────┐    │    │
│  │  │  StatefulSet: aof-daemon (3 replicas)          │    │    │
│  │  │                                                 │    │    │
│  │  │  aof-daemon-0                                   │    │    │
│  │  │  ├─ Liveness:  GET /health                     │    │    │
│  │  │  ├─ Readiness: GET /ready                      │    │    │
│  │  │  ├─ Metrics:   GET /metrics (scraped)          │    │    │
│  │  │  └─ PVC: data (10Gi), checkpoints (5Gi)        │    │    │
│  │  │                                                 │    │    │
│  │  │  aof-daemon-1 (same)                           │    │    │
│  │  │  aof-daemon-2 (same)                           │    │    │
│  │  └────────────────────────────────────────────────┘    │    │
│  │                        │                                 │    │
│  │  ┌────────────────────┼────────────────────┐           │    │
│  │  │                    │                    │           │    │
│  │  ▼                    ▼                    ▼           │    │
│  │  Service            Redis          ConfigMap           │    │
│  │  (ClusterIP)      (StatefulSet)    (config)            │    │
│  └────────────────────────────────────────────────────────┘    │
│                        │                                        │
│  ┌────────────────────┼────────────────────────────────┐      │
│  │  Monitoring Stack  │                                 │      │
│  │                    ▼                                 │      │
│  │  Prometheus ───> Grafana                            │      │
│  │  (scrapes /metrics)  (dashboards)                   │      │
│  └──────────────────────────────────────────────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                        │
                        │ HTTPS
                        ▼
               ┌────────────────┐
               │    Ingress     │
               │ (nginx/Traefik)│
               │ - TLS          │
               │ - Rate limit   │
               └────────────────┘
                        │
                        ▼
                   Internet
```

---

## 12. Conclusion

### What We Know

1. **Binary Distribution**: Solid foundation with GitHub Actions, install script, multi-platform builds
2. **Container Deployment**: Production-ready Dockerfile with multi-stage build, health checks
3. **Systemd Integration**: Basic service file exists, needs security hardening
4. **State Management**: Session persistence exists, needs versioned migration strategy

### What We Need

1. **Health Endpoints**: Implement `/health`, `/ready`, `/metrics` in `serve.rs`
2. **Structured Logging**: Migrate to `tracing-subscriber` with JSON format
3. **Prometheus Metrics**: Define and instrument key metrics
4. **Graceful Shutdown**: Save state before exit
5. **Kubernetes Manifests**: Create production-ready StatefulSet, Services, ConfigMaps
6. **Upgrade Strategy**: Document procedures, test rollback scenarios

### Success Criteria for Phase 8

- [ ] Health check endpoints return valid responses
- [ ] Prometheus successfully scrapes `/metrics`
- [ ] Logs are structured JSON with contextual fields
- [ ] Graceful shutdown saves all session state
- [ ] Systemd service runs with security hardening
- [ ] Docker Compose stack runs with monitoring
- [ ] Kubernetes StatefulSet deploys successfully
- [ ] Upgrade from v1.0.0 → v1.1.0 tested (with rollback)
- [ ] Documentation complete (deployment guide, runbooks)

**Estimated Effort:** 2-3 weeks for core implementation + 1 week for testing and documentation

---

**Document Status:** Ready for planning
**Next Action:** Create Phase 8 PLAN based on this research
