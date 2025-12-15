# AOF Production Deployment Guide

> Comprehensive guide for deploying the Agentic Ops Framework (AOF) in production environments

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Deployment Options](#deployment-options)
3. [Platform Setup](#platform-setup)
4. [Configuration](#configuration)
5. [Security](#security)
6. [Monitoring](#monitoring)
7. [Scaling](#scaling)
8. [Troubleshooting](#troubleshooting)

---

## 1. Prerequisites

### System Requirements

**Minimum Requirements:**
- CPU: 2 cores (4 cores recommended)
- RAM: 4 GB (8 GB recommended)
- Disk: 10 GB available space
- OS: Linux (Ubuntu 20.04+, RHEL 8+), macOS 11+, Windows Server 2019+

**Network Requirements:**
- Outbound HTTPS (443) for LLM provider APIs
- Inbound ports for webhook servers (default: 8080)
- Low latency connection to LLM providers (&lt;100ms recommended)

### Dependencies

**Core Dependencies:**
```bash
# Rust toolchain (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup update

# Build tools (Linux)
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Build tools (macOS)
xcode-select --install

# Optional: Docker & Docker Compose
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh
```

**Runtime Dependencies:**
- OpenSSL 1.1.1+ or 3.0+
- glibc 2.31+ (Linux)
- libc++ (macOS)

### API Keys Required

1. **LLM Provider Keys** (at least one):
   - Anthropic: `ANTHROPIC_API_KEY`
   - OpenAI: `OPENAI_API_KEY`
   - AWS Bedrock: AWS credentials configured

2. **Platform Integration Keys** (optional):
   - WhatsApp Business: `WHATSAPP_ACCESS_TOKEN`, `WHATSAPP_VERIFY_TOKEN`
   - Telegram: `TELEGRAM_BOT_TOKEN`
   - Slack: `SLACK_BOT_TOKEN`, `SLACK_SIGNING_SECRET`
   - Discord: `DISCORD_BOT_TOKEN`, `DISCORD_PUBLIC_KEY`

3. **MCP Server Access** (optional):
   - Server-specific authentication tokens
   - OAuth credentials for cloud-based MCP servers

---

## 2. Deployment Options

### Option A: Standalone Binary

**Best for:** Simple deployments, single-server setups, development

```bash
# Build release binary
cargo build --release --workspace

# Install aofctl globally
cargo install --path crates/aofctl

# Verify installation
aofctl --version

# Run agent
aofctl run --config /etc/aof/agent.yaml \
  --input "Deploy application to production"
```

**Production Setup:**
```bash
# Create system user
sudo useradd -r -s /bin/false aof

# Install binary
sudo cp target/release/aofctl /usr/local/bin/
sudo chmod +x /usr/local/bin/aofctl

# Create directories
sudo mkdir -p /etc/aof /var/lib/aof /var/log/aof
sudo chown -R aof:aof /var/lib/aof /var/log/aof

# Create systemd service
sudo tee /etc/systemd/system/aof-agent.service > /dev/null <<'EOF'
[Unit]
Description=AOF Agent Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=aof
Group=aof
ExecStart=/usr/local/bin/aofctl run --config /etc/aof/agent.yaml
Restart=always
RestartSec=10
StandardOutput=append:/var/log/aof/agent.log
StandardError=append:/var/log/aof/agent.err

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/aof /var/log/aof

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable aof-agent
sudo systemctl start aof-agent
```

### Option B: Docker Container

**Best for:** Containerized environments, Kubernetes, cloud platforms

**Dockerfile:**
```dockerfile
# Multi-stage build for minimal image size
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /build

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY . .

# Build release
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false aof

# Copy binaries
COPY --from=builder /build/target/release/aofctl /usr/local/bin/
COPY --from=builder /build/target/release/aof-triggers /usr/local/bin/

# Set permissions
RUN chown aof:aof /usr/local/bin/aofctl /usr/local/bin/aof-triggers

USER aof
WORKDIR /app

EXPOSE 8080

CMD ["aofctl", "run", "--config", "/config/agent.yaml"]
```

**Build and Run:**
```bash
# Build image
docker build -t aof:latest .

# Run container
docker run -d \
  --name aof-agent \
  --restart unless-stopped \
  -v $(pwd)/config:/config:ro \
  -v aof-data:/data \
  -e ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY}" \
  -e RUST_LOG=info \
  -p 8080:8080 \
  aof:latest

# Check logs
docker logs -f aof-agent
```

**Docker Compose:**
```yaml
version: '3.8'

services:
  aof-agent:
    build: .
    container_name: aof-agent
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./config:/config:ro
      - aof-data:/data
      - aof-logs:/logs
    environment:
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - RUST_LOG=info
      - AOF_MEMORY_BACKEND=redis
      - REDIS_URL=redis://redis:6379
    depends_on:
      - redis
    networks:
      - aof-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    container_name: aof-redis
    restart: unless-stopped
    volumes:
      - redis-data:/data
    networks:
      - aof-network
    command: redis-server --appendonly yes

  aof-triggers:
    build: .
    container_name: aof-triggers
    restart: unless-stopped
    ports:
      - "8081:8080"
    volumes:
      - ./config:/config:ro
    environment:
      - WHATSAPP_ACCESS_TOKEN=${WHATSAPP_ACCESS_TOKEN}
      - TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN}
      - SLACK_BOT_TOKEN=${SLACK_BOT_TOKEN}
      - DISCORD_BOT_TOKEN=${DISCORD_BOT_TOKEN}
    networks:
      - aof-network
    command: ["/usr/local/bin/aof-triggers"]

volumes:
  aof-data:
  aof-logs:
  redis-data:

networks:
  aof-network:
    driver: bridge
```

### Option C: Kubernetes Deployment

**Best for:** Large-scale deployments, high availability, auto-scaling

**Namespace:**
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: aof-system
  labels:
    app.kubernetes.io/name: aof
```

**ConfigMap:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: aof-config
  namespace: aof-system
data:
  agent.yaml: |
    apiVersion: aof.dev/v1
    kind: Agent
    metadata:
      name: production-agent
    spec:
      model:
        provider: anthropic
        model: claude-3-5-sonnet-20241022
      tools:
        - name: kubectl
          type: mcp
          config:
            command: npx
            args: ["-y", "kubectl-mcp"]
      memory:
        backend: redis
        config:
          url: redis://aof-redis:6379
```

**Secret:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: aof-secrets
  namespace: aof-system
type: Opaque
stringData:
  anthropic-api-key: "sk-ant-..."
  openai-api-key: "sk-..."
  whatsapp-token: "..."
  telegram-token: "..."
```

**Deployment:**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aof-agent
  namespace: aof-system
  labels:
    app: aof-agent
spec:
  replicas: 3
  selector:
    matchLabels:
      app: aof-agent
  template:
    metadata:
      labels:
        app: aof-agent
    spec:
      serviceAccountName: aof-agent
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000
      containers:
      - name: agent
        image: aof:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 8080
          name: http
          protocol: TCP
        env:
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: aof-secrets
              key: anthropic-api-key
        - name: RUST_LOG
          value: "info"
        - name: AOF_MEMORY_BACKEND
          value: "redis"
        - name: REDIS_URL
          value: "redis://aof-redis:6379"
        volumeMounts:
        - name: config
          mountPath: /config
          readOnly: true
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: aof-config
      - name: data
        persistentVolumeClaim:
          claimName: aof-data
```

**Service:**
```yaml
apiVersion: v1
kind: Service
metadata:
  name: aof-agent
  namespace: aof-system
spec:
  type: ClusterIP
  selector:
    app: aof-agent
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
    name: http
```

**HorizontalPodAutoscaler:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: aof-agent
  namespace: aof-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: aof-agent
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**Deploy to Kubernetes:**
```bash
# Apply manifests
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/hpa.yaml

# Verify deployment
kubectl get pods -n aof-system
kubectl logs -f deployment/aof-agent -n aof-system

# Check scaling
kubectl get hpa -n aof-system -w
```

### Option D: Desktop App Distribution

**Best for:** End-user deployments, local development, GUI access

**Build Desktop App:**
```bash
# Build Tauri application
cd crates/aof-gui
npm install
npm run tauri build

# Output locations:
# - macOS: target/release/bundle/dmg/AOF.dmg
# - Windows: target/release/bundle/msi/AOF.msi
# - Linux: target/release/bundle/deb/aof_*.deb
```

**macOS Distribution:**
```bash
# Sign application
codesign --deep --force --verify --verbose \
  --sign "Developer ID Application: Your Name" \
  --options runtime \
  target/release/bundle/macos/AOF.app

# Notarize
xcrun notarytool submit target/release/bundle/dmg/AOF.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID"

# Staple ticket
xcrun stapler staple target/release/bundle/dmg/AOF.dmg
```

**Windows Distribution:**
```powershell
# Sign MSI
signtool sign /f cert.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 target/release/bundle/msi/AOF.msi
```

**Linux Distribution:**
```bash
# Build AppImage
cd crates/aof-gui
cargo install cargo-appimage
cargo appimage

# Build Flatpak
flatpak-builder --repo=repo build-dir io.aof.App.yaml
```

---

## 3. Platform Setup

### WhatsApp Business API Setup

**Prerequisites:**
- Meta Business Account
- WhatsApp Business Account
- Phone number (not used on WhatsApp)

**Step 1: Create WhatsApp Business App**
```bash
# Via Meta Developer Console
# 1. Go to https://developers.facebook.com/apps
# 2. Create App > Business > WhatsApp
# 3. Add WhatsApp product
# 4. Get test number or add your own
```

**Step 2: Configure Webhook**
```yaml
# config/whatsapp.yaml
webhook_url: "https://your-domain.com/webhooks/whatsapp"
verify_token: "your-random-verify-token-here"
access_token: "EAAxxxxxxxxxxxxxxxx"
phone_number_id: "123456789"
```

**Step 3: Set Environment Variables**
```bash
export WHATSAPP_ACCESS_TOKEN="EAAxxxxxxxxxxxxxxxx"
export WHATSAPP_VERIFY_TOKEN="your-random-verify-token-here"
export WHATSAPP_PHONE_NUMBER_ID="123456789"
```

**Step 4: Verify Webhook**
```bash
# Meta will send GET request to verify
# Your server must return challenge parameter
curl "https://your-domain.com/webhooks/whatsapp?hub.mode=subscribe&hub.challenge=CHALLENGE&hub.verify_token=your-random-verify-token-here"
```

**Step 5: Test Integration**
```bash
# Start webhook server
cargo run -p aof-triggers

# Send test message via WhatsApp
# Server logs should show incoming webhook
```

### Telegram Bot Creation

**Step 1: Create Bot via BotFather**
```bash
# 1. Open Telegram and search for @BotFather
# 2. Send /newbot
# 3. Follow prompts to set name and username
# 4. Save the bot token
```

**Step 2: Configure Bot**
```bash
# Set webhook
curl -X POST "https://api.telegram.org/bot<TOKEN>/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-domain.com/webhooks/telegram",
    "secret_token": "your-secret-token"
  }'

# Verify webhook
curl "https://api.telegram.org/bot<TOKEN>/getWebhookInfo"
```

**Step 3: Environment Variables**
```bash
export TELEGRAM_BOT_TOKEN="1234567890:ABCdefGHIjklMNOpqrsTUVwxyz"
export TELEGRAM_WEBHOOK_SECRET="your-secret-token"
```

**Step 4: Bot Commands**
```bash
# Set bot commands
curl -X POST "https://api.telegram.org/bot<TOKEN>/setMyCommands" \
  -H "Content-Type: application/json" \
  -d '{
    "commands": [
      {"command": "start", "description": "Start the agent"},
      {"command": "help", "description": "Show help"},
      {"command": "status", "description": "Check agent status"}
    ]
  }'
```

### Slack App Configuration

**Step 1: Create Slack App**
```bash
# 1. Go to https://api.slack.com/apps
# 2. Click "Create New App"
# 3. Choose "From scratch"
# 4. Enter app name and workspace
```

**Step 2: Configure OAuth & Permissions**
```yaml
# Required Bot Token Scopes:
- chat:write
- chat:write.public
- commands
- im:history
- im:read
- im:write
- users:read
```

**Step 3: Enable Event Subscriptions**
```bash
# Request URL: https://your-domain.com/webhooks/slack
# Subscribe to bot events:
- message.im
- app_mention
```

**Step 4: Install App to Workspace**
```bash
# Get tokens from OAuth & Permissions page
export SLACK_BOT_TOKEN="xoxb-..."
export SLACK_SIGNING_SECRET="..."
export SLACK_APP_TOKEN="xapp-..."  # For Socket Mode
```

**Step 5: Create Slash Commands (Optional)**
```bash
# Command: /aof
# Request URL: https://your-domain.com/webhooks/slack/commands
# Short Description: "Run AOF agent"
```

### Discord Bot Setup

**Step 1: Create Discord Application**
```bash
# 1. Go to https://discord.com/developers/applications
# 2. Click "New Application"
# 3. Enter application name
# 4. Go to "Bot" section
# 5. Click "Add Bot"
```

**Step 2: Configure Bot Permissions**
```yaml
# Required Permissions:
- Send Messages
- Read Message History
- Use Slash Commands
- Embed Links
- Attach Files
```

**Step 3: Get Bot Token**
```bash
export DISCORD_BOT_TOKEN="MTxxxxxxxxxxxxxxxxxx.xxxxxx.xxxxxxxxxxxxxxxxxxxxxxxxxxx"
export DISCORD_PUBLIC_KEY="xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
export DISCORD_APPLICATION_ID="123456789012345678"
```

**Step 4: Register Slash Commands**
```bash
curl -X POST \
  "https://discord.com/api/v10/applications/${DISCORD_APPLICATION_ID}/commands" \
  -H "Authorization: Bot ${DISCORD_BOT_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "aof",
    "description": "Execute AOF agent task",
    "options": [
      {
        "name": "task",
        "description": "Task description",
        "type": 3,
        "required": true
      }
    ]
  }'
```

**Step 5: Invite Bot to Server**
```bash
# Generate OAuth2 URL with bot scope and permissions
# https://discord.com/oauth2/authorize?client_id=CLIENT_ID&scope=bot+applications.commands&permissions=PERMISSIONS
```

---

## 4. Configuration

### Environment Variables

**Core Variables:**
```bash
# LLM Provider Configuration
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."

# AWS Bedrock (alternative to env vars)
export AWS_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# Application Settings
export RUST_LOG="info,aof_core=debug"
export AOF_CONFIG_PATH="/etc/aof/agent.yaml"
export AOF_DATA_DIR="/var/lib/aof"

# Memory Backend
export AOF_MEMORY_BACKEND="redis"  # Options: memory, redis, sled, file
export REDIS_URL="redis://localhost:6379"

# Webhook Server
export AOF_WEBHOOK_HOST="0.0.0.0"
export AOF_WEBHOOK_PORT="8080"

# Platform Tokens
export WHATSAPP_ACCESS_TOKEN="..."
export TELEGRAM_BOT_TOKEN="..."
export SLACK_BOT_TOKEN="..."
export DISCORD_BOT_TOKEN="..."
```

**Production .env File:**
```bash
# /etc/aof/.env
ANTHROPIC_API_KEY=sk-ant-api03-xxxxx
RUST_LOG=info,aof_core=debug,aof_runtime=info
AOF_MEMORY_BACKEND=redis
REDIS_URL=redis://:password@localhost:6379/0
AOF_WEBHOOK_HOST=0.0.0.0
AOF_WEBHOOK_PORT=8080
WHATSAPP_ACCESS_TOKEN=EAAxxxxx
TELEGRAM_BOT_TOKEN=123456:ABCxxx
```

### YAML Configuration

**Agent Configuration:**
```yaml
# /etc/aof/agent.yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: production-devops-agent
  labels:
    environment: production
    team: platform
spec:
  # Model configuration
  model:
    provider: anthropic  # Options: anthropic, openai, bedrock
    model: claude-3-5-sonnet-20241022
    temperature: 0.7
    max_tokens: 4096
    timeout_seconds: 300

  # Tool configuration
  tools:
    - name: kubectl
      type: mcp
      config:
        command: npx
        args: ["-y", "kubectl-mcp"]
        transport: stdio

    - name: aws-cli
      type: mcp
      config:
        command: npx
        args: ["-y", "aws-mcp"]
        transport: stdio

    - name: terraform
      type: custom
      config:
        command: terraform
        allowed_commands: ["plan", "apply", "destroy"]

  # Memory configuration
  memory:
    backend: redis
    config:
      url: redis://localhost:6379
      db: 0
      pool_size: 10
      timeout_seconds: 5
      ttl_seconds: 3600

  # Execution limits
  limits:
    max_iterations: 50
    max_execution_time_seconds: 1800
    max_memory_mb: 2048

  # Retry configuration
  retry:
    max_attempts: 3
    backoff_seconds: 5
    exponential: true
```

**Multi-Agent Configuration:**
```yaml
# /etc/aof/agents.yaml
apiVersion: aof.dev/v1
kind: AgentList
agents:
  - metadata:
      name: k8s-operator
    spec:
      model:
        provider: anthropic
        model: claude-3-5-sonnet-20241022
      tools: [kubectl, helm]

  - metadata:
      name: aws-architect
    spec:
      model:
        provider: bedrock
        model: anthropic.claude-3-5-sonnet-20241022-v2:0
      tools: [aws-cli, terraform]

  - metadata:
      name: security-auditor
    spec:
      model:
        provider: openai
        model: gpt-4
      tools: [trivy, kubesec]
```

### Secrets Management

**Option 1: Environment Variables**
```bash
# Load from secure vault
export ANTHROPIC_API_KEY=$(vault kv get -field=key secret/aof/anthropic)
export OPENAI_API_KEY=$(vault kv get -field=key secret/aof/openai)
```

**Option 2: Kubernetes Secrets**
```bash
# Create secret from file
kubectl create secret generic aof-secrets \
  --from-file=anthropic-key=/path/to/key \
  --from-file=openai-key=/path/to/key \
  -n aof-system

# Use in deployment
env:
- name: ANTHROPIC_API_KEY
  valueFrom:
    secretKeyRef:
      name: aof-secrets
      key: anthropic-key
```

**Option 3: AWS Secrets Manager**
```rust
// Load at runtime
use aws_config::load_from_env;
use aws_sdk_secretsmanager::Client;

async fn load_secret(name: &str) -> String {
    let config = load_from_env().await;
    let client = Client::new(&config);
    let resp = client
        .get_secret_value()
        .secret_id(name)
        .send()
        .await
        .unwrap();
    resp.secret_string().unwrap().to_string()
}
```

**Option 4: HashiCorp Vault**
```bash
# Initialize Vault
vault kv put secret/aof/anthropic key="sk-ant-..."
vault kv put secret/aof/openai key="sk-..."

# Create policy
vault policy write aof-policy - <<EOF
path "secret/data/aof/*" {
  capabilities = ["read"]
}
EOF

# Generate token
vault token create -policy=aof-policy
```

### MCP Server Setup

**Stdio Transport:**
```yaml
tools:
  - name: filesystem
    type: mcp
    config:
      command: npx
      args: ["-y", "@modelcontextprotocol/server-filesystem", "/allowed/path"]
      transport: stdio
```

**HTTP Transport:**
```yaml
tools:
  - name: remote-api
    type: mcp
    config:
      url: "https://mcp.example.com"
      transport: http
      headers:
        Authorization: "Bearer ${MCP_API_KEY}"
```

**SSE Transport:**
```yaml
tools:
  - name: streaming-api
    type: mcp
    config:
      url: "https://mcp.example.com/sse"
      transport: sse
      reconnect: true
      reconnect_delay_ms: 1000
```

---

## 5. Security

### Webhook Signature Verification

**WhatsApp Signature Verification:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_whatsapp_signature(
    payload: &[u8],
    signature: &str,
    app_secret: &str,
) -> bool {
    type HmacSha256 = Hmac<Sha256>;

    let expected = signature.strip_prefix("sha256=").unwrap_or(signature);

    let mut mac = HmacSha256::new_from_slice(app_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);

    let result = mac.finalize();
    let code_bytes = result.into_bytes();

    hex::encode(code_bytes) == expected
}
```

**Telegram Signature Verification:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_telegram_signature(
    secret_token: &str,
    header_token: &str,
) -> bool {
    secret_token == header_token
}
```

**Slack Signature Verification:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_slack_signature(
    signing_secret: &str,
    timestamp: &str,
    body: &str,
    signature: &str,
) -> bool {
    type HmacSha256 = Hmac<Sha256>;

    let sig_basestring = format!("v0:{}:{}", timestamp, body);

    let mut mac = HmacSha256::new_from_slice(signing_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(sig_basestring.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    let computed = format!("v0={}", hex::encode(code_bytes));

    computed == signature
}
```

**Discord Signature Verification:**
```rust
use ed25519_dalek::{PublicKey, Signature, Verifier};

fn verify_discord_signature(
    public_key: &str,
    signature: &str,
    timestamp: &str,
    body: &str,
) -> bool {
    let message = format!("{}{}", timestamp, body);

    let public_key_bytes = hex::decode(public_key).unwrap();
    let signature_bytes = hex::decode(signature).unwrap();

    let public_key = PublicKey::from_bytes(&public_key_bytes).unwrap();
    let signature = Signature::from_bytes(&signature_bytes).unwrap();

    public_key.verify(message.as_bytes(), &signature).is_ok()
}
```

### Rate Limiting

**Application-Level Rate Limiting:**
```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

// Create rate limiter: 100 requests per minute
let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());
let limiter = RateLimiter::direct(quota);

// Check rate limit
if limiter.check().is_err() {
    return Err("Rate limit exceeded");
}
```

**Nginx Rate Limiting:**
```nginx
http {
    limit_req_zone $binary_remote_addr zone=webhook_limit:10m rate=10r/s;

    server {
        location /webhooks/ {
            limit_req zone=webhook_limit burst=20 nodelay;
            proxy_pass http://aof-backend;
        }
    }
}
```

**Kubernetes Rate Limiting (Ingress):**
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: aof-ingress
  annotations:
    nginx.ingress.kubernetes.io/limit-rps: "10"
    nginx.ingress.kubernetes.io/limit-connections: "100"
spec:
  rules:
  - host: aof.example.com
    http:
      paths:
      - path: /webhooks
        pathType: Prefix
        backend:
          service:
            name: aof-agent
            port:
              number: 80
```

### API Key Rotation

**Automated Rotation Script:**
```bash
#!/bin/bash
# rotate-keys.sh

# Rotate Anthropic API key
NEW_KEY=$(vault kv get -field=key secret/aof/anthropic-new)
kubectl set env deployment/aof-agent \
  ANTHROPIC_API_KEY="${NEW_KEY}" \
  -n aof-system

# Wait for rollout
kubectl rollout status deployment/aof-agent -n aof-system

# Archive old key
vault kv put secret/aof/anthropic-archived \
  key="$(vault kv get -field=key secret/aof/anthropic)" \
  rotated_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Update current key
vault kv put secret/aof/anthropic key="${NEW_KEY}"

echo "API key rotation complete"
```

**Rotation Schedule (Cron):**
```bash
# /etc/cron.d/aof-key-rotation
0 2 1 * * /usr/local/bin/rotate-keys.sh >> /var/log/aof/key-rotation.log 2>&1
```

### Network Policies

**Kubernetes NetworkPolicy:**
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: aof-agent-network-policy
  namespace: aof-system
spec:
  podSelector:
    matchLabels:
      app: aof-agent
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  # Allow DNS
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: UDP
      port: 53
  # Allow Redis
  - to:
    - podSelector:
        matchLabels:
          app: aof-redis
    ports:
    - protocol: TCP
      port: 6379
  # Allow HTTPS to LLM providers
  - to:
    - podSelector: {}
    ports:
    - protocol: TCP
      port: 443
```

**Firewall Rules (iptables):**
```bash
# Allow incoming webhook traffic
iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

# Allow outgoing HTTPS to LLM providers
iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT

# Block all other incoming traffic
iptables -A INPUT -j DROP
```

---

## 6. Monitoring

### Logging Configuration

**Structured Logging:**
```bash
# Environment variable configuration
export RUST_LOG="info,aof_core=debug,aof_runtime=info,aof_llm=debug"

# JSON logging for production
export RUST_LOG_FORMAT="json"
```

**Log Rotation:**
```bash
# /etc/logrotate.d/aof
/var/log/aof/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0644 aof aof
    sharedscripts
    postrotate
        systemctl reload aof-agent > /dev/null 2>&1 || true
    endscript
}
```

**Centralized Logging (Fluent Bit):**
```yaml
# fluent-bit.conf
[SERVICE]
    Flush        5
    Daemon       Off
    Log_Level    info

[INPUT]
    Name              tail
    Path              /var/log/aof/*.log
    Parser            json
    Tag               aof.*

[OUTPUT]
    Name              es
    Match             aof.*
    Host              elasticsearch
    Port              9200
    Index             aof-logs
    Type              _doc
```

**Kubernetes Logging (Fluentd):**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
  namespace: kube-system
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/containers/aof-agent*.log
      pos_file /var/log/fluentd-aof.log.pos
      tag kubernetes.aof
      <parse>
        @type json
        time_key time
        time_format %Y-%m-%dT%H:%M:%S.%NZ
      </parse>
    </source>

    <match kubernetes.aof>
      @type elasticsearch
      host elasticsearch.logging.svc.cluster.local
      port 9200
      index_name aof-logs
      type_name _doc
    </match>
```

### Metrics Collection

**Prometheus Metrics:**
```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();

    static ref AGENT_EXECUTIONS: Counter = Counter::new(
        "aof_agent_executions_total",
        "Total number of agent executions"
    ).unwrap();

    static ref EXECUTION_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "aof_execution_duration_seconds",
            "Agent execution duration in seconds"
        )
    ).unwrap();

    static ref LLM_REQUESTS: Counter = Counter::new(
        "aof_llm_requests_total",
        "Total number of LLM API requests"
    ).unwrap();

    static ref LLM_TOKENS: Counter = Counter::new(
        "aof_llm_tokens_total",
        "Total number of tokens consumed"
    ).unwrap();
}

// Export metrics endpoint
async fn metrics_handler() -> impl Responder {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

**Prometheus Scrape Config:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'aof-agents'
    kubernetes_sd_configs:
      - role: pod
        namespaces:
          names:
            - aof-system
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app]
        regex: aof-agent
        action: keep
      - source_labels: [__meta_kubernetes_pod_ip]
        target_label: __address__
        replacement: ${1}:8080
```

**ServiceMonitor (Prometheus Operator):**
```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: aof-agent
  namespace: aof-system
spec:
  selector:
    matchLabels:
      app: aof-agent
  endpoints:
  - port: http
    path: /metrics
    interval: 30s
```

### Alerting Setup

**Prometheus Alerts:**
```yaml
# alerts.yml
groups:
- name: aof-alerts
  interval: 30s
  rules:
  - alert: AOFAgentDown
    expr: up{job="aof-agents"} == 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "AOF agent is down"
      description: "AOF agent {{ $labels.instance }} has been down for 5 minutes"

  - alert: AOFHighErrorRate
    expr: rate(aof_agent_errors_total[5m]) > 0.1
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "High error rate in AOF agent"
      description: "Error rate is {{ $value }} errors/sec on {{ $labels.instance }}"

  - alert: AOFHighLatency
    expr: histogram_quantile(0.95, aof_execution_duration_seconds_bucket) > 30
    for: 15m
    labels:
      severity: warning
    annotations:
      summary: "High execution latency"
      description: "P95 latency is {{ $value }}s on {{ $labels.instance }}"

  - alert: AOFHighTokenUsage
    expr: rate(aof_llm_tokens_total[1h]) > 100000
    for: 1h
    labels:
      severity: info
    annotations:
      summary: "High LLM token usage"
      description: "Token usage is {{ $value }} tokens/sec"
```

**AlertManager Config:**
```yaml
# alertmanager.yml
global:
  resolve_timeout: 5m

route:
  group_by: ['alertname', 'cluster']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h
  receiver: 'default'
  routes:
  - match:
      severity: critical
    receiver: pagerduty
  - match:
      severity: warning
    receiver: slack

receivers:
- name: 'default'
  email_configs:
  - to: 'team@example.com'
    from: 'alerts@example.com'
    smarthost: 'smtp.gmail.com:587'
    auth_username: 'alerts@example.com'
    auth_password: 'password'

- name: 'slack'
  slack_configs:
  - api_url: 'https://hooks.slack.com/services/xxx'
    channel: '#alerts'
    title: 'AOF Alert: {{ .GroupLabels.alertname }}'
    text: '{{ range .Alerts }}{{ .Annotations.description }}{{ end }}'

- name: 'pagerduty'
  pagerduty_configs:
  - service_key: 'xxx'
    description: '{{ .GroupLabels.alertname }}'
```

### Health Checks

**Application Health Check:**
```rust
use axum::{Router, routing::get};

async fn health_check() -> &'static str {
    "OK"
}

async fn readiness_check() -> Result<&'static str, StatusCode> {
    // Check dependencies
    if redis_available().await && llm_provider_available().await {
        Ok("READY")
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

let app = Router::new()
    .route("/health", get(health_check))
    .route("/ready", get(readiness_check));
```

**Kubernetes Probes:**
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  successThreshold: 1
  failureThreshold: 3
```

**External Health Monitoring:**
```bash
# UptimeRobot-style check
*/5 * * * * curl -f https://aof.example.com/health || echo "Health check failed" | mail -s "AOF Down" alerts@example.com
```

---

## 7. Scaling

### Horizontal Scaling

**Docker Compose Scale:**
```bash
# Scale to 5 replicas
docker-compose up -d --scale aof-agent=5

# With load balancer
docker-compose -f docker-compose.yml -f docker-compose.scale.yml up -d
```

**Kubernetes HPA:**
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: aof-agent
  namespace: aof-system
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: aof-agent
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: aof_execution_duration_seconds
      target:
        type: AverageValue
        averageValue: "10"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30
      - type: Pods
        value: 2
        periodSeconds: 30
      selectPolicy: Max
```

**KEDA (Event-Driven Autoscaling):**
```yaml
apiVersion: keda.sh/v1alpha1
kind: ScaledObject
metadata:
  name: aof-agent-scaler
  namespace: aof-system
spec:
  scaleTargetRef:
    name: aof-agent
  minReplicaCount: 3
  maxReplicaCount: 20
  triggers:
  - type: prometheus
    metadata:
      serverAddress: http://prometheus:9090
      metricName: aof_queue_depth
      threshold: '10'
      query: sum(aof_pending_tasks)
```

### Load Balancing

**Nginx Load Balancer:**
```nginx
upstream aof_backend {
    least_conn;
    server aof-1:8080 max_fails=3 fail_timeout=30s;
    server aof-2:8080 max_fails=3 fail_timeout=30s;
    server aof-3:8080 max_fails=3 fail_timeout=30s;
    keepalive 32;
}

server {
    listen 80;
    server_name aof.example.com;

    location / {
        proxy_pass http://aof_backend;
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_connect_timeout 60s;
        proxy_send_timeout 300s;
        proxy_read_timeout 300s;
    }
}
```

**HAProxy Configuration:**
```haproxy
global
    maxconn 4096
    daemon

defaults
    mode http
    timeout connect 10s
    timeout client 300s
    timeout server 300s

frontend aof_frontend
    bind *:80
    default_backend aof_backend

backend aof_backend
    balance leastconn
    option httpchk GET /health
    http-check expect status 200
    server aof-1 10.0.1.10:8080 check inter 5s rise 2 fall 3
    server aof-2 10.0.1.11:8080 check inter 5s rise 2 fall 3
    server aof-3 10.0.1.12:8080 check inter 5s rise 2 fall 3
```

### Database Configuration

**Redis Cluster:**
```yaml
# docker-compose.redis-cluster.yml
version: '3.8'

services:
  redis-node-1:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000 --appendonly yes
    volumes:
      - redis-1:/data

  redis-node-2:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000 --appendonly yes
    volumes:
      - redis-2:/data

  redis-node-3:
    image: redis:7-alpine
    command: redis-server --cluster-enabled yes --cluster-config-file nodes.conf --cluster-node-timeout 5000 --appendonly yes
    volumes:
      - redis-3:/data

  redis-cluster-init:
    image: redis:7-alpine
    command: redis-cli --cluster create redis-node-1:6379 redis-node-2:6379 redis-node-3:6379 --cluster-replicas 0 --cluster-yes
    depends_on:
      - redis-node-1
      - redis-node-2
      - redis-node-3

volumes:
  redis-1:
  redis-2:
  redis-3:
```

**Redis Sentinel (High Availability):**
```yaml
# sentinel.conf
sentinel monitor aof-master 10.0.1.10 6379 2
sentinel down-after-milliseconds aof-master 5000
sentinel parallel-syncs aof-master 1
sentinel failover-timeout aof-master 10000
```

**Application Redis Config:**
```yaml
memory:
  backend: redis
  config:
    # Sentinel configuration
    sentinels:
      - host: sentinel-1
        port: 26379
      - host: sentinel-2
        port: 26379
      - host: sentinel-3
        port: 26379
    master_name: aof-master
    db: 0
    pool_size: 20
    timeout_seconds: 5
```

### Caching Strategies

**Redis Caching Layer:**
```rust
use redis::{Client, Commands};

struct CachedLLMProvider {
    provider: Box<dyn Model>,
    cache: Client,
}

impl CachedLLMProvider {
    async fn generate(&self, request: &ModelRequest) -> AofResult<ModelResponse> {
        let cache_key = format!("llm:{}:{}", request.model, hash(request.messages));

        // Check cache
        if let Ok(cached) = self.cache.get::<_, String>(&cache_key) {
            return Ok(serde_json::from_str(&cached)?);
        }

        // Generate response
        let response = self.provider.generate(request).await?;

        // Cache response (TTL: 1 hour)
        let _: () = self.cache.set_ex(
            &cache_key,
            serde_json::to_string(&response)?,
            3600
        )?;

        Ok(response)
    }
}
```

**CDN Caching (CloudFlare):**
```nginx
# Add cache headers for static content
location /static/ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# Cache API responses
location /api/public/ {
    proxy_cache api_cache;
    proxy_cache_valid 200 10m;
    proxy_cache_key "$request_uri";
    add_header X-Cache-Status $upstream_cache_status;
}
```

---

## 8. Troubleshooting

### Common Issues

**Issue 1: Agent Execution Timeout**
```
Error: Agent execution timed out after 300 seconds
```

**Solution:**
```yaml
# Increase timeout in agent.yaml
spec:
  limits:
    max_execution_time_seconds: 1800  # 30 minutes
```

**Issue 2: Memory Exhausted**
```
Error: Cannot allocate memory
```

**Solution:**
```bash
# Check memory usage
docker stats

# Increase container memory
docker run -m 4g aof:latest

# Kubernetes
resources:
  limits:
    memory: "4Gi"
```

**Issue 3: Redis Connection Failed**
```
Error: Connection refused (redis://localhost:6379)
```

**Solution:**
```bash
# Check Redis status
redis-cli ping

# Verify connection
telnet localhost 6379

# Check network
docker network inspect aof-network

# Update connection string
export REDIS_URL="redis://redis-host:6379"
```

**Issue 4: LLM API Rate Limit**
```
Error: Rate limit exceeded (429)
```

**Solution:**
```rust
// Implement exponential backoff
use tokio::time::{sleep, Duration};

for attempt in 0..5 {
    match provider.generate(request).await {
        Ok(response) => return Ok(response),
        Err(e) if e.is_rate_limit() => {
            let delay = 2_u64.pow(attempt) * 1000;
            sleep(Duration::from_millis(delay)).await;
        }
        Err(e) => return Err(e),
    }
}
```

**Issue 5: Webhook Signature Verification Failed**
```
Error: Invalid signature
```

**Solution:**
```bash
# Verify secret is correct
echo $WHATSAPP_VERIFY_TOKEN

# Check webhook payload
tail -f /var/log/aof/webhooks.log

# Test signature locally
curl -X POST http://localhost:8080/webhooks/whatsapp \
  -H "X-Hub-Signature-256: sha256=..." \
  -d @payload.json
```

### Debug Mode

**Enable Debug Logging:**
```bash
# Full debug
export RUST_LOG="debug"

# Selective debug
export RUST_LOG="info,aof_core=debug,aof_llm=trace"

# With timestamps
export RUST_LOG="info,aof_core=debug"
export RUST_LOG_STYLE="always"
```

**Interactive Debugging:**
```bash
# Run with debugger
rust-lldb target/debug/aofctl

# Set breakpoints
b aof_core::agent::execute
run --config agent.yaml

# Inspect variables
p request
p response
```

**Trace Network Calls:**
```bash
# Enable HTTP tracing
export RUST_LOG="reqwest=trace"

# Capture with tcpdump
sudo tcpdump -i any -w aof-traffic.pcap port 443

# Analyze with wireshark
wireshark aof-traffic.pcap
```

### Log Analysis

**Parse JSON Logs:**
```bash
# Extract errors
jq 'select(.level == "ERROR")' /var/log/aof/agent.log

# Count by error type
jq -r '.error_type' /var/log/aof/agent.log | sort | uniq -c

# Filter by time range
jq 'select(.timestamp > "2024-01-01T00:00:00Z")' /var/log/aof/agent.log
```

**Elasticsearch Queries:**
```json
{
  "query": {
    "bool": {
      "must": [
        { "match": { "app": "aof-agent" } },
        { "range": { "@timestamp": { "gte": "now-1h" } } }
      ],
      "should": [
        { "match": { "level": "ERROR" } },
        { "match": { "level": "WARN" } }
      ]
    }
  },
  "aggs": {
    "errors_by_type": {
      "terms": { "field": "error_type.keyword" }
    }
  }
}
```

**Loki LogQL:**
```logql
{app="aof-agent"}
  |= "ERROR"
  | json
  | level="ERROR"
  | line_format "{{.timestamp}} {{.error_type}}: {{.message}}"
```

### Performance Tuning

**Profile CPU Usage:**
```bash
# Install flamegraph
cargo install flamegraph

# Profile application
sudo flamegraph target/release/aofctl run --config agent.yaml

# View flamegraph.svg in browser
```

**Memory Profiling:**
```bash
# Use valgrind
valgrind --tool=massif --massif-out-file=massif.out \
  target/release/aofctl run --config agent.yaml

# Analyze
ms_print massif.out
```

**Benchmark Performance:**
```bash
# Load testing with k6
k6 run - <<EOF
import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};

export default function() {
  let res = http.post('http://localhost:8080/webhooks/test',
    JSON.stringify({ message: 'test' }),
    { headers: { 'Content-Type': 'application/json' } }
  );
  check(res, { 'status is 200': (r) => r.status === 200 });
}
EOF
```

**Database Performance:**
```bash
# Redis slowlog
redis-cli slowlog get 10

# Monitor commands
redis-cli monitor

# Benchmark
redis-benchmark -h localhost -p 6379 -c 50 -n 10000
```

**LLM Provider Latency:**
```rust
use std::time::Instant;

let start = Instant::now();
let response = provider.generate(request).await?;
let duration = start.elapsed();

metrics::histogram!("aof_llm_latency_seconds", duration.as_secs_f64());
```

---

## Appendix

### Quick Reference

**Essential Commands:**
```bash
# Build
cargo build --release --workspace

# Run agent
aofctl run --config agent.yaml --input "task"

# Run webhook server
aof-triggers --config triggers.yaml

# Check logs
journalctl -u aof-agent -f

# Health check
curl http://localhost:8080/health

# Metrics
curl http://localhost:8080/metrics
```

**Environment Variables:**
```bash
ANTHROPIC_API_KEY        # Anthropic API key
OPENAI_API_KEY          # OpenAI API key
RUST_LOG                # Logging level
AOF_MEMORY_BACKEND      # Memory backend (memory, redis, sled, file)
REDIS_URL               # Redis connection string
AOF_WEBHOOK_PORT        # Webhook server port
```

**Default Ports:**
- `8080` - Main application / webhook server
- `8081` - Metrics endpoint
- `6379` - Redis
- `9090` - Prometheus
- `3000` - Grafana

### Support Resources

- **Documentation:** https://github.com/your-org/aof/docs
- **Issues:** https://github.com/your-org/aof/issues
- **Discord:** https://discord.gg/aof
- **Email:** support@aof.dev

---

**Last Updated:** 2024-12-10
**Version:** 1.0.0
