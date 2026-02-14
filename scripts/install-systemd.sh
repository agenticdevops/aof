#!/usr/bin/env bash
#
# AOF Systemd Installation Script
#
# This script installs and configures the AOF daemon as a systemd service
# with security hardening and production best practices.
#
# Usage:
#   sudo ./install-systemd.sh
#
# Prerequisites:
#   - Linux system with systemd
#   - Root/sudo access
#   - aofctl binary in PATH or ./target/release/aofctl

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   log_error "This script must be run as root (use sudo)"
   exit 1
fi

log_info "Installing AOF Daemon as systemd service..."

# Step 1: Create aof system user
if id "aof" &>/dev/null; then
    log_info "User 'aof' already exists"
else
    log_info "Creating system user 'aof'..."
    useradd -r -m -d /opt/aof -s /bin/bash aof
fi

# Step 2: Create directories
log_info "Creating directories..."
mkdir -p /opt/aof
mkdir -p /etc/aof
mkdir -p /var/lib/aof/{sessions,checkpoints}
mkdir -p /var/log/aof

# Set ownership
chown -R aof:aof /opt/aof
chown -R aof:aof /var/lib/aof
chown -R aof:aof /var/log/aof
chown -R root:aof /etc/aof
chmod 750 /etc/aof

log_info "Directories created:"
log_info "  /opt/aof - Application directory"
log_info "  /etc/aof - Configuration directory"
log_info "  /var/lib/aof - Data directory (sessions, checkpoints)"
log_info "  /var/log/aof - Log directory"

# Step 3: Copy binary
log_info "Installing aofctl binary..."
if [[ -f "./target/release/aofctl" ]]; then
    cp ./target/release/aofctl /usr/local/bin/aofctl
    log_info "Copied aofctl from ./target/release/aofctl"
elif command -v aofctl &> /dev/null; then
    AOFCTL_PATH=$(command -v aofctl)
    cp "$AOFCTL_PATH" /usr/local/bin/aofctl
    log_info "Copied aofctl from $AOFCTL_PATH"
else
    log_error "aofctl binary not found. Please build it first or ensure it's in PATH."
    exit 1
fi

chmod 755 /usr/local/bin/aofctl

# Step 4: Create example config if not exists
if [[ ! -f /etc/aof/daemon.yaml ]]; then
    log_info "Creating example configuration..."
    cat > /etc/aof/daemon.yaml <<'EOF'
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
    token_limits:
      max_overhead_percent: 30.0
      auto_degrade: true
      recovery_threshold: 20.0
EOF
    chown root:aof /etc/aof/daemon.yaml
    chmod 640 /etc/aof/daemon.yaml
    log_info "Created /etc/aof/daemon.yaml (edit to customize)"
else
    log_info "Configuration already exists at /etc/aof/daemon.yaml"
fi

# Step 5: Create environment file template
if [[ ! -f /etc/aof/daemon.env ]]; then
    log_info "Creating environment template..."
    cat > /etc/aof/daemon.env <<'EOF'
# AOF Daemon Environment Variables
#
# LLM Provider API Keys (uncomment and set as needed)
# ANTHROPIC_API_KEY=sk-ant-...
# OPENAI_API_KEY=sk-...
#
# Messaging Platform Credentials
# SLACK_BOT_TOKEN=xoxb-...
# SLACK_SIGNING_SECRET=...
# DISCORD_BOT_TOKEN=...
# TELEGRAM_BOT_TOKEN=...
#
# Additional settings
# RUST_LOG=debug,aofctl=debug
EOF
    chown root:aof /etc/aof/daemon.env
    chmod 640 /etc/aof/daemon.env
    log_warn "Set API keys in /etc/aof/daemon.env before starting the service"
else
    log_info "Environment file already exists at /etc/aof/daemon.env"
fi

# Step 6: Install systemd service
log_info "Installing systemd service unit..."
cp ./scripts/aof-daemon.service /etc/systemd/system/aof-daemon.service
chmod 644 /etc/systemd/system/aof-daemon.service

# Step 7: Reload systemd
log_info "Reloading systemd daemon..."
systemctl daemon-reload

# Step 8: Enable service
log_info "Enabling aof-daemon service..."
systemctl enable aof-daemon

log_info "${GREEN}Installation complete!${NC}"
echo
echo "Next steps:"
echo "  1. Edit /etc/aof/daemon.yaml to configure your deployment"
echo "  2. Set API keys in /etc/aof/daemon.env (e.g., ANTHROPIC_API_KEY)"
echo "  3. Start the service: sudo systemctl start aof-daemon"
echo "  4. Check status: sudo systemctl status aof-daemon"
echo "  5. View logs: sudo journalctl -u aof-daemon -f"
echo
echo "Service management:"
echo "  Start:   sudo systemctl start aof-daemon"
echo "  Stop:    sudo systemctl stop aof-daemon"
echo "  Restart: sudo systemctl restart aof-daemon"
echo "  Status:  sudo systemctl status aof-daemon"
echo "  Logs:    sudo journalctl -u aof-daemon -f"
echo
echo "Health check: http://localhost:8080/health"
echo "Metrics: http://localhost:8080/metrics"
