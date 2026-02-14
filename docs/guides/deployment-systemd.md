# Deploying AOF with Systemd

This guide walks through deploying the AOF daemon as a systemd service on Linux servers.

## Prerequisites

- Linux server with systemd (Ubuntu 20.04+, Debian 11+, RHEL 8+, etc.)
- Root/sudo access
- AOF binary (`aofctl`) built or downloaded

## Quick Install

```bash
# Download and run installation script
curl -sSL https://docs.aof.sh/install.sh | bash
sudo ./scripts/install-systemd.sh
```

The script will:
1. Create `aof` system user
2. Set up directories (`/opt/aof`, `/etc/aof`, `/var/lib/aof`, `/var/log/aof`)
3. Install `aofctl` binary to `/usr/local/bin`
4. Create example configuration
5. Install and enable systemd service

## Manual Installation

### Step 1: Build or Download Binary

```bash
# Option A: Build from source
git clone https://github.com/agenticdevops/aof
cd aof
cargo build --release
sudo cp target/release/aofctl /usr/local/bin/aofctl

# Option B: Download pre-built binary
curl -L https://github.com/agenticdevops/aof/releases/latest/download/aofctl-linux-x86_64 -o aofctl
chmod +x aofctl
sudo mv aofctl /usr/local/bin/aofctl
```

### Step 2: Create System User

```bash
sudo useradd -r -m -d /opt/aof -s /bin/bash aof
```

### Step 3: Create Directories

```bash
sudo mkdir -p /opt/aof
sudo mkdir -p /etc/aof
sudo mkdir -p /var/lib/aof/{sessions,checkpoints}
sudo mkdir -p /var/log/aof

sudo chown -R aof:aof /opt/aof
sudo chown -R aof:aof /var/lib/aof
sudo chown -R aof:aof /var/log/aof
sudo chown -R root:aof /etc/aof
sudo chmod 750 /etc/aof
```

### Step 4: Create Configuration

Create `/etc/aof/daemon.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: DaemonConfig
metadata:
  name: aof-daemon
  labels:
    environment: production

spec:
  server:
    port: 8080
    host: 0.0.0.0
    cors: true
    timeout_secs: 30

  agents:
    directory: /opt/aof/agents
    watch: true

  flows:
    directory: /opt/aof/flows
    enabled: true
    watch: true

  runtime:
    max_concurrent_tasks: 10
    task_timeout_secs: 300
    max_tasks_per_user: 3

  decision_log:
    enabled: true
    path: /var/lib/aof/decisions.jsonl

  coordination:
    enabled: true
    mode: full
    heartbeat:
      frequency_secs: 60
      timeout_secs: 120
```

Set permissions:

```bash
sudo chown root:aof /etc/aof/daemon.yaml
sudo chmod 640 /etc/aof/daemon.yaml
```

### Step 5: Create Environment File

Create `/etc/aof/daemon.env`:

```bash
# LLM Provider API Keys
ANTHROPIC_API_KEY=sk-ant-your-key-here
# OPENAI_API_KEY=sk-your-key-here

# Messaging Platform Credentials
# SLACK_BOT_TOKEN=xoxb-your-token
# SLACK_SIGNING_SECRET=your-secret
# DISCORD_BOT_TOKEN=your-token
# TELEGRAM_BOT_TOKEN=your-token

# Logging (optional, defaults to info)
# RUST_LOG=debug,aofctl=debug
```

Set permissions:

```bash
sudo chown root:aof /etc/aof/daemon.env
sudo chmod 640 /etc/aof/daemon.env
```

### Step 6: Install Systemd Service

Copy `scripts/aof-daemon.service` to `/etc/systemd/system/`:

```bash
sudo cp scripts/aof-daemon.service /etc/systemd/system/aof-daemon.service
sudo chmod 644 /etc/systemd/system/aof-daemon.service
sudo systemctl daemon-reload
```

### Step 7: Enable and Start Service

```bash
sudo systemctl enable aof-daemon
sudo systemctl start aof-daemon
```

## Service Management

### Check Status

```bash
sudo systemctl status aof-daemon
```

Output:

```
● aof-daemon.service - AOF Daemon - Agentic Ops Framework
     Loaded: loaded (/etc/systemd/system/aof-daemon.service; enabled; vendor preset: enabled)
     Active: active (running) since Mon 2024-02-12 10:30:45 UTC; 2h 15min ago
       Docs: https://docs.aof.sh
   Main PID: 12345 (aofctl)
      Tasks: 24 (limit: 4096)
     Memory: 256.3M (limit: 2.0G)
        CPU: 1min 23.456s
     CGroup: /system.slice/aof-daemon.service
             └─12345 /usr/local/bin/aofctl serve --config /etc/aof/daemon.yaml ...
```

### View Logs

```bash
# Follow logs in real-time
sudo journalctl -u aof-daemon -f

# View last 100 lines
sudo journalctl -u aof-daemon -n 100

# View logs since 1 hour ago
sudo journalctl -u aof-daemon --since "1 hour ago"

# View logs with timestamps
sudo journalctl -u aof-daemon -o short-iso

# Export to file
sudo journalctl -u aof-daemon --since today > aof-logs.txt
```

### Start/Stop/Restart

```bash
sudo systemctl start aof-daemon
sudo systemctl stop aof-daemon
sudo systemctl restart aof-daemon
```

### Reload Configuration

After editing `/etc/aof/daemon.yaml`:

```bash
sudo systemctl restart aof-daemon
```

### Disable Service

```bash
sudo systemctl stop aof-daemon
sudo systemctl disable aof-daemon
```

## Monitoring

### Health Check

```bash
curl http://localhost:8080/health
```

Response:

```json
{
  "status": "ok",
  "version": "0.4.0-beta",
  "uptime_seconds": 8145,
  "git_commit": "abc123def"
}
```

### Readiness Check

```bash
curl http://localhost:8080/ready
```

### Prometheus Metrics

```bash
curl http://localhost:8080/metrics
```

### Integration with Prometheus

Add to `/etc/prometheus/prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'aof-daemon'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

Reload Prometheus:

```bash
sudo systemctl reload prometheus
```

## Security Hardening

The systemd service includes 15+ security directives:

### Filesystem Isolation

- `ProtectSystem=strict` - Entire filesystem read-only except explicitly allowed paths
- `ProtectHome=true` - Home directories inaccessible
- `PrivateTmp=true` - Private /tmp directory
- `ReadWritePaths=/var/lib/aof /var/log/aof` - Only these paths writable
- `ReadOnlyPaths=/etc/aof` - Config directory read-only

### Kernel Protection

- `ProtectKernelTunables=true` - Cannot modify kernel parameters
- `ProtectKernelModules=true` - Cannot load kernel modules
- `ProtectKernelLogs=true` - Cannot access kernel logs
- `ProtectControlGroups=true` - cgroups read-only

### Privilege Restriction

- `NoNewPrivileges=true` - Cannot gain new privileges
- `RestrictSUIDSGID=true` - Cannot create SUID/SGID files
- `LockPersonality=true` - Execution domain locked

### Network Restriction

- `RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX` - Only TCP/UDP/Unix sockets

### System Call Filtering

- `RestrictNamespaces=true` - Cannot create namespaces
- `RestrictRealtime=true` - Cannot use realtime scheduling

## Troubleshooting

### Service Won't Start

**Check logs**:

```bash
sudo journalctl -u aof-daemon -n 50
```

**Common issues**:

1. **Missing API key**:
   ```
   ERROR: No LLM API key configured
   ```
   Solution: Add `ANTHROPIC_API_KEY` to `/etc/aof/daemon.env`

2. **Port already in use**:
   ```
   ERROR: Failed to bind to 0.0.0.0:8080: Address already in use
   ```
   Solution: Change port in `/etc/aof/daemon.yaml` or stop conflicting service

3. **Permission denied on /var/lib/aof**:
   ```
   ERROR: Failed to create session persistence directory
   ```
   Solution: Fix ownership: `sudo chown -R aof:aof /var/lib/aof`

### High Memory Usage

**Check current usage**:

```bash
sudo systemctl status aof-daemon | grep Memory
```

**Adjust limit** in `/etc/systemd/system/aof-daemon.service`:

```ini
MemoryMax=4G  # Increase from 2G
```

Reload and restart:

```bash
sudo systemctl daemon-reload
sudo systemctl restart aof-daemon
```

### Service Crashes on Startup

**Check for core dumps**:

```bash
coredumpctl list | grep aofctl
```

**Enable debug logging**:

Add to `/etc/aof/daemon.env`:

```bash
RUST_LOG=debug,aofctl=debug,aof_runtime=debug
RUST_BACKTRACE=1
```

Restart and check logs:

```bash
sudo systemctl restart aof-daemon
sudo journalctl -u aof-daemon -f
```

## Upgrade Procedure

### Option A: In-place Upgrade (Recommended)

```bash
# Download new binary
curl -L https://github.com/agenticdevops/aof/releases/download/v0.4.1/aofctl-linux-x86_64 -o aofctl
chmod +x aofctl

# Stop service
sudo systemctl stop aof-daemon

# Backup old binary
sudo mv /usr/local/bin/aofctl /usr/local/bin/aofctl.old

# Install new binary
sudo mv aofctl /usr/local/bin/aofctl

# Start service
sudo systemctl start aof-daemon

# Verify
curl http://localhost:8080/health
```

### Option B: Rolling Upgrade (Zero Downtime)

Requires load balancer and multiple instances:

```bash
# Upgrade instance 1
sudo systemctl stop aof-daemon@1
# ... upgrade binary ...
sudo systemctl start aof-daemon@1

# Wait for health check to pass
curl http://localhost:8081/health

# Repeat for instance 2, 3, etc.
```

### Rollback

```bash
sudo systemctl stop aof-daemon
sudo mv /usr/local/bin/aofctl.old /usr/local/bin/aofctl
sudo systemctl start aof-daemon
```

## Backup and Restore

### Backup

```bash
# Create backup directory
sudo mkdir -p /backup/aof/$(date +%Y%m%d)

# Backup configuration
sudo cp -r /etc/aof /backup/aof/$(date +%Y%m%d)/

# Backup data (sessions, checkpoints, decisions)
sudo tar czf /backup/aof/$(date +%Y%m%d)/data.tar.gz /var/lib/aof

# Backup logs
sudo tar czf /backup/aof/$(date +%Y%m%d)/logs.tar.gz /var/log/aof
```

### Restore

```bash
# Stop service
sudo systemctl stop aof-daemon

# Restore configuration
sudo cp -r /backup/aof/20240212/aof /etc/

# Restore data
sudo tar xzf /backup/aof/20240212/data.tar.gz -C /

# Fix permissions
sudo chown -R aof:aof /var/lib/aof

# Start service
sudo systemctl start aof-daemon
```

## Performance Tuning

### Increase File Descriptors

Edit `/etc/systemd/system/aof-daemon.service`:

```ini
LimitNOFILE=131072  # Increase from 65536
```

### Increase Process Limit

```ini
LimitNPROC=8192  # Increase from 4096
```

### CPU Affinity

Pin to specific CPUs:

```ini
CPUAffinity=0-3  # Use cores 0-3
```

### I/O Priority

```ini
IOSchedulingClass=realtime
IOSchedulingPriority=0
```

Reload and restart:

```bash
sudo systemctl daemon-reload
sudo systemctl restart aof-daemon
```

## Next Steps

- [Configure agents and flows](./configuration.md)
- [Set up Prometheus monitoring](./monitoring.md)
- [Enable SSL/TLS](./ssl-tls.md)
- [Deploy on Kubernetes](./deployment-kubernetes.md)
