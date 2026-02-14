# Production Deployment Guide

This guide covers deploying AOF Mission Control in development and production environments.

## Table of Contents

- [Development Setup](#development-setup)
- [Production Build](#production-build)
- [Docker Deployment](#docker-deployment)
- [Systemd Service](#systemd-service)
- [Reverse Proxy (nginx)](#reverse-proxy-nginx)
- [Troubleshooting](#troubleshooting)

## Development Setup

### Prerequisites

- Rust 1.70+ (`rustup update`)
- Node.js 18+ and npm
- Git

### Running Locally

**Terminal 1: Build and start Rust daemon**
```bash
cd /path/to/aof

# Build release binary
cargo build -p aofctl --release

# Start daemon (serves HTTP + WebSocket + API)
./target/release/aofctl serve \
  --workspace-root . \
  --static-dir ./web-ui/dist \
  --port 8080
```

**Terminal 2: React dev server (with hot reload)**
```bash
cd web-ui

# Install dependencies (first time only)
npm install

# Start dev server
npm run dev
# Opens http://localhost:5173
# Proxies API requests to localhost:8080
```

### Development Workflow

1. Edit React components in `web-ui/src/`
2. Browser auto-reloads on save (Vite HMR)
3. Edit Rust code in `crates/`
4. Restart `aofctl serve` to see changes
5. WebSocket reconnects automatically

## Production Build

### Single-Daemon Deployment (Recommended)

Production mode serves React build and APIs from single Rust daemon on port 8080. No Node.js required.

**Step 1: Build React frontend**
```bash
cd web-ui
npm run build
# Creates optimized bundle in web-ui/dist/
# Bundle size: ~500KB gzipped
```

**Step 2: Build Rust backend**
```bash
cd ..
cargo build -p aofctl --release --locked
# Binary: target/release/aofctl (~50MB)
```

**Step 3: Run single daemon**
```bash
./target/release/aofctl serve \
  --workspace-root /var/lib/aof \
  --static-dir ./web-ui/dist \
  --port 8080
```

**Step 4: Access UI**
```
http://localhost:8080/          # React app
http://localhost:8080/api/config/agents  # Config API
ws://localhost:8080/ws          # WebSocket events
```

### Configuration File

Create `serve-config.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig

metadata:
  name: mission-control

spec:
  server:
    port: 8080
    host: 0.0.0.0
    cors: true
    timeout_secs: 30

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
    max_tasks_per_user: 3

  decision_log:
    enabled: true
    path: /var/lib/aof/decisions.jsonl
```

Run with config:
```bash
aofctl serve --config serve-config.yaml --static-dir ./web-ui/dist
```

## Docker Deployment

### Dockerfile

Create `Dockerfile`:

```dockerfile
# Build stage: React frontend
FROM node:18-alpine AS web-builder
WORKDIR /app/web-ui
COPY web-ui/package*.json ./
RUN npm ci --production
COPY web-ui/ .
RUN npm run build

# Build stage: Rust backend
FROM rust:1.76-alpine AS rust-builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
RUN cargo build -p aofctl --release --locked

# Runtime stage
FROM alpine:3.19
RUN apk add --no-cache ca-certificates
WORKDIR /app

# Copy Rust binary
COPY --from=rust-builder /app/target/release/aofctl /usr/local/bin/

# Copy React build
COPY --from=web-builder /app/web-ui/dist /app/web-ui/dist

# Create workspace directory
RUN mkdir -p /var/lib/aof
COPY AGENTS.md TOOLS.md /var/lib/aof/

# Expose port
EXPOSE 8080

# Run daemon
CMD ["aofctl", "serve", \
     "--workspace-root", "/var/lib/aof", \
     "--static-dir", "/app/web-ui/dist", \
     "--port", "8080"]
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  mission-control:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./AGENTS.md:/var/lib/aof/AGENTS.md:ro
      - ./TOOLS.md:/var/lib/aof/TOOLS.md:ro
      - aof-data:/var/lib/aof
    environment:
      RUST_LOG: info
    restart: unless-stopped

volumes:
  aof-data:
```

**Build and run:**
```bash
docker-compose build
docker-compose up -d

# View logs
docker-compose logs -f mission-control

# Stop
docker-compose down
```

## Systemd Service

For Ubuntu/Debian production servers.

### Service File

Create `/etc/systemd/system/aof-mission-control.service`:

```ini
[Unit]
Description=AOF Mission Control Daemon
After=network.target

[Service]
Type=simple
User=aof
Group=aof
WorkingDirectory=/opt/aof
ExecStart=/opt/aof/aofctl serve \
    --workspace-root /var/lib/aof \
    --static-dir /opt/aof/web-ui/dist \
    --port 8080
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal
SyslogIdentifier=aof-mission-control

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/aof

[Install]
WantedBy=multi-user.target
```

### Setup Steps

```bash
# Create user
sudo useradd -r -s /bin/false aof

# Create directories
sudo mkdir -p /opt/aof /var/lib/aof
sudo chown aof:aof /var/lib/aof

# Copy binary and static files
sudo cp target/release/aofctl /opt/aof/
sudo cp -r web-ui/dist /opt/aof/web-ui/

# Copy config
sudo cp AGENTS.md TOOLS.md /var/lib/aof/
sudo chown aof:aof /var/lib/aof/*

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable aof-mission-control
sudo systemctl start aof-mission-control

# Check status
sudo systemctl status aof-mission-control
sudo journalctl -u aof-mission-control -f
```

## Reverse Proxy (nginx)

For HTTPS and domain name support.

### nginx Configuration

Create `/etc/nginx/sites-available/mission-control`:

```nginx
upstream aof_backend {
    server 127.0.0.1:8080;
}

server {
    listen 80;
    server_name mission-control.example.com;

    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name mission-control.example.com;

    # SSL certificates (Let's Encrypt recommended)
    ssl_certificate /etc/letsencrypt/live/mission-control.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/mission-control.example.com/privkey.pem;

    # WebSocket support
    location /ws {
        proxy_pass http://aof_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;  # 24h for long-lived connections
    }

    # API routes
    location /api/ {
        proxy_pass http://aof_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Static files (React app)
    location / {
        proxy_pass http://aof_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
            proxy_pass http://aof_backend;
            proxy_cache_valid 200 1y;
            add_header Cache-Control "public, immutable";
        }
    }
}
```

**Enable site:**
```bash
sudo ln -s /etc/nginx/sites-available/mission-control /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## Troubleshooting

### Common Issues

#### Port Already in Use

**Error:** `Failed to bind to 0.0.0.0:8080: address already in use`

**Solution:**
```bash
# Find process using port
sudo lsof -i :8080
# or
sudo netstat -tulpn | grep :8080

# Kill process or choose different port
aofctl serve --port 8081
```

#### Config File Not Found

**Error:** `Failed to read AGENTS.md at /var/lib/aof/AGENTS.md: No such file or directory`

**Solution:**
```bash
# Check workspace root
ls -la /var/lib/aof/

# Verify path matches --workspace-root flag
aofctl serve --workspace-root /path/to/actual/workspace
```

#### YAML Parse Error

**Error:** `Failed to parse AGENTS.md at ./AGENTS.md Field: agents[0].skills Error: invalid type: string, expected a sequence`

**Solution:**
- YAML syntax error in AGENTS.md or TOOLS.md
- The error shows exact field path (`agents[0].skills`)
- Common issue: `skills: kubectl` (string) should be `skills: [kubectl]` (array)
- Validate YAML: `yamllint AGENTS.md`

#### WebSocket Connection Failed

**Error:** Browser console shows `WebSocket connection to 'ws://localhost:8080/ws' failed`

**Solution:**
```bash
# Check daemon is running
curl http://localhost:8080/health

# Check WebSocket route
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" http://localhost:8080/ws

# If behind nginx, ensure proxy_http_version 1.1 and upgrade headers set
```

#### React App Shows Blank Page

**Error:** Browser shows white screen, no errors in console

**Solution:**
```bash
# Rebuild React app
cd web-ui
rm -rf dist node_modules
npm install
npm run build

# Verify dist/ created
ls -la dist/

# Restart daemon with correct static dir
aofctl serve --static-dir ./web-ui/dist
```

#### 404 on Refresh (SPA Routing)

**Error:** Navigating to `/agents` works, but refreshing gives 404

**Solution:**
- This indicates static fallback not configured
- Check `ServeDir::fallback()` in serve.rs is set to `index.html`
- Verify `--static-dir` points to correct React build directory

### Performance Tuning

#### High Memory Usage

Default event buffer: 1000 events in memory

Reduce buffer in `serve.rs`:
```rust
let event_bus = Arc::new(EventBroadcaster::new(500)); // 500 events instead of 1000
```

#### Slow API Responses

Increase worker threads:
```bash
# Set Tokio runtime threads
TOKIO_WORKER_THREADS=8 aofctl serve
```

### Logging

Enable debug logging:
```bash
RUST_LOG=debug aofctl serve

# Or specific modules
RUST_LOG=aofctl=debug,aof_triggers=debug aofctl serve
```

View structured logs:
```bash
# With systemd
sudo journalctl -u aof-mission-control -f --output=json-pretty

# Docker
docker-compose logs -f mission-control | jq
```

## Architecture Diagram

```
┌─────────────┐
│   Browser   │
└──────┬──────┘
       │ HTTP/WS
       ▼
┌─────────────────────────────────────┐
│      nginx (reverse proxy)          │  Port 443 (HTTPS)
│  - SSL termination                  │
│  - WebSocket upgrade                │
│  - Static asset caching             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│    AOF Daemon (aofctl serve)        │  Port 8080
│  ┌─────────────────────────────┐   │
│  │  Axum HTTP Server           │   │
│  ├─────────────────────────────┤   │
│  │  /api/config/*              │   │  Config API
│  │  /webhook/:platform         │   │  Trigger webhooks
│  │  /ws                        │   │  WebSocket events
│  │  /*                         │   │  Static files (React)
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  EventBroadcaster           │   │  Real-time events
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  TriggerHandler             │   │  Platform adapters
│  └─────────────────────────────┘   │
└──────────────┬──────────────────────┘
               │
               ▼
       ┌──────────────┐
       │  Filesystem  │
       │  - AGENTS.md │
       │  - TOOLS.md  │
       └──────────────┘
```

## Security Considerations

1. **CORS:** Default `Access-Control-Allow-Origin: *` is for development. Production should restrict to same origin.
2. **HTTPS:** Always use HTTPS in production (nginx with Let's Encrypt).
3. **Authentication:** Current version has no built-in auth. Add auth layer (OAuth2, JWT) in nginx or reverse proxy.
4. **Rate Limiting:** Add rate limiting in nginx to prevent abuse.
5. **Secrets:** Never commit AGENTS.md/TOOLS.md with secrets. Use environment variables or secret managers.

## Next Steps

After deployment:
1. Configure monitoring (Prometheus + Grafana recommended)
2. Set up log aggregation (ELK/Loki)
3. Configure backups for `/var/lib/aof` (decision logs, session state)
4. Add alerting for daemon crashes
5. Implement health checks and auto-restart

## Support

- Documentation: https://docs.aof.sh
- Issues: https://github.com/agenticdevops/aof/issues
- Discussions: https://github.com/agenticdevops/aof/discussions
