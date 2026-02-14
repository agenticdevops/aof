# Deploying AOF with Docker

Quick guide for running AOF daemon in Docker containers.

## Quick Start

```bash
# Build image
docker build -t aof:latest .

# Run daemon
docker run -d \
  --name aof-daemon \
  -p 8080:8080 \
  -e ANTHROPIC_API_KEY=sk-ant-your-key \
  -v $(pwd)/agents:/app/agents \
  -v aof-data:/var/lib/aof \
  --restart unless-stopped \
  aof:latest
```

## Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  aof-daemon:
    build: .
    image: aof:latest
    container_name: aof-daemon
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info,aofctl=info,aof_runtime=info
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - SLACK_BOT_TOKEN=${SLACK_BOT_TOKEN}
      - SLACK_SIGNING_SECRET=${SLACK_SIGNING_SECRET}
    volumes:
      - ./agents:/app/agents:ro
      - ./flows:/app/flows:ro
      - ./config:/app/config:ro
      - aof-data:/var/lib/aof
      - aof-checkpoints:/app/checkpoints
    healthcheck:
      test: ["CMD", "wget", "--no-verbose", "--tries=1", "--spider", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      start_period: 10s
      retries: 3
    security_opt:
      - no-new-privileges:true
    read_only: true
    tmpfs:
      - /tmp:mode=1777,size=100m
    user: "1000:1000"

volumes:
  aof-data:
  aof-checkpoints:
```

Start:

```bash
docker-compose up -d
```

## Full Stack with Monitoring

```yaml
version: '3.8'

services:
  aof-daemon:
    # ... (same as above)

  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-data:/var/lib/grafana

volumes:
  aof-data:
  aof-checkpoints:
  prometheus-data:
  grafana-data:
```

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'aof-daemon'
    static_configs:
      - targets: ['aof-daemon:8080']
    metrics_path: '/metrics'
```

## Environment Variables

```bash
# LLM Providers
ANTHROPIC_API_KEY=sk-ant-your-key
OPENAI_API_KEY=sk-your-key

# Messaging Platforms
SLACK_BOT_TOKEN=xoxb-your-token
SLACK_SIGNING_SECRET=your-secret
DISCORD_BOT_TOKEN=your-token
TELEGRAM_BOT_TOKEN=your-token

# Logging
RUST_LOG=info,aofctl=info,aof_runtime=info

# Directories
AOF_AGENTS_DIR=/app/agents
AOF_CONFIG_DIR=/app/config
AOF_DATA_DIR=/var/lib/aof
```

## Volume Management

### Named Volumes (Recommended)

```bash
docker volume create aof-data
docker volume create aof-checkpoints

docker run -d \
  -v aof-data:/var/lib/aof \
  -v aof-checkpoints:/app/checkpoints \
  aof:latest
```

### Bind Mounts (Development)

```bash
docker run -d \
  -v $(pwd)/agents:/app/agents \
  -v $(pwd)/data:/var/lib/aof \
  aof:latest
```

## Monitoring

### Health Check

```bash
docker exec aof-daemon curl http://localhost:8080/health
```

### Metrics

```bash
docker exec aof-daemon curl http://localhost:8080/metrics
```

### Logs

```bash
# Follow logs
docker logs -f aof-daemon

# JSON logs
docker logs aof-daemon | jq

# Export logs
docker logs aof-daemon > aof.log
```

## Upgrading

```bash
# Pull new image
docker pull aof:v0.4.1

# Stop old container
docker stop aof-daemon

# Remove old container
docker rm aof-daemon

# Start new container (volumes persist)
docker run -d \
  --name aof-daemon \
  -p 8080:8080 \
  -e ANTHROPIC_API_KEY=sk-ant-your-key \
  -v aof-data:/var/lib/aof \
  --restart unless-stopped \
  aof:v0.4.1
```

## Backup and Restore

### Backup

```bash
# Backup data volume
docker run --rm \
  -v aof-data:/data \
  -v $(pwd)/backup:/backup \
  alpine tar czf /backup/aof-data-$(date +%Y%m%d).tar.gz /data

# Backup using docker cp
docker cp aof-daemon:/var/lib/aof ./backup/
```

### Restore

```bash
# Restore data volume
docker run --rm \
  -v aof-data:/data \
  -v $(pwd)/backup:/backup \
  alpine tar xzf /backup/aof-data-20240212.tar.gz -C /
```

## Resource Limits

```bash
docker run -d \
  --name aof-daemon \
  --memory="2g" \
  --memory-swap="2g" \
  --cpus="2" \
  --pids-limit=4096 \
  aof:latest
```

## Security

```bash
docker run -d \
  --name aof-daemon \
  --security-opt no-new-privileges:true \
  --cap-drop=ALL \
  --read-only \
  --user 1000:1000 \
  --tmpfs /tmp:mode=1777,size=100m \
  aof:latest
```

## Troubleshooting

### Container Won't Start

```bash
docker logs aof-daemon
docker inspect aof-daemon
```

### Permission Denied

```bash
# Fix volume permissions
docker run --rm -v aof-data:/data alpine chown -R 1000:1000 /data
```

### Network Issues

```bash
docker network inspect bridge
docker exec aof-daemon ping google.com
```

## Next Steps

- [Configure agents](./configuration.md)
- [Set up reverse proxy](./nginx-proxy.md)
- [Deploy with Docker Swarm](./docker-swarm.md)
