# Gateway Troubleshooting Guide

## Common Issues

### "Invalid bot token" error

**Symptom:** Gateway fails to start with authentication error

**Causes:**
- Token not set in environment variable
- Token copied incorrectly (trailing spaces, wrong token type)
- Token revoked/expired

**Solutions:**
1. Verify environment variable is set:
```bash
echo $SLACK_BOT_TOKEN
```

2. Check token type:
   - Slack bot token starts with `xoxb-`
   - Slack app token starts with `xapp-`
   - Discord bot token is alphanumeric
   - Telegram bot token format: `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`

3. Regenerate token in platform console:
   - **Slack**: https://api.slack.com/apps → Your App → OAuth & Permissions
   - **Discord**: https://discord.com/developers/applications → Your App → Bot → Reset Token
   - **Telegram**: Message @BotFather → `/token` → Your Bot

### "Missing required environment variables" error

**Symptom:** Config loading fails with missing variable error

**Example error:**
```
Missing required environment variables: SLACK_BOT_TOKEN, DISCORD_BOT_TOKEN
```

**Solutions:**
1. Check .env file exists and is loaded:
```bash
ls -la .env
cat .env  # Verify variables are defined
```

2. Verify variable name matches config:
```yaml
config:
  bot_token: "${SLACK_BOT_TOKEN}"  # Must match exactly
```

3. Export variable in shell:
```bash
export SLACK_BOT_TOKEN="xoxb-your-token"
export SLACK_APP_TOKEN="xapp-your-token"
```

4. For production, use secret management:
```bash
# Kubernetes
kubectl create secret generic gateway-secrets \
  --from-literal=SLACK_BOT_TOKEN="xoxb-..." \
  --from-literal=DISCORD_BOT_TOKEN="..."

# AWS Secrets Manager
export SLACK_BOT_TOKEN=$(aws secretsmanager get-secret-value \
  --secret-id slack-bot-token --query SecretString --output text)
```

### Messages not received in Slack

**Symptom:** Bot is online but doesn't respond to messages

**Causes:**
- Socket Mode not enabled
- Bot not invited to channel
- Insufficient bot scopes

**Solutions:**
1. Enable Socket Mode:
   - Go to: https://api.slack.com/apps → Your App → Socket Mode
   - Toggle "Enable Socket Mode" to ON
   - Generate App-Level Token with `connections:write` scope

2. Invite bot to channel:
```
/invite @your-bot-name
```

3. Add required scopes:
   - Go to: OAuth & Permissions → Scopes
   - Add Bot Token Scopes:
     - `channels:history` - Read messages
     - `chat:write` - Send messages
     - `reactions:read` - Read reactions (optional)
   - Reinstall app to workspace after adding scopes

4. Verify bot user ID matches config:
```yaml
config:
  bot_user_id: "U01234567"  # Must match actual bot user ID
```

Find bot user ID:
```bash
curl -H "Authorization: Bearer xoxb-your-token" \
  https://slack.api/auth.test | jq '.user_id'
```

### Rate limit errors (429)

**Symptom:** Messages fail with "rate limited" error

**Example error:**
```
2026-02-13 10:23:45 WARN Failed to broadcast to channel: Rate limit exceeded (retry after 60s)
```

**Causes:**
- Too many messages sent in short period
- Burst size exceeded
- Platform rate limit hit

**Solutions:**
1. Increase burst_size in config (if legitimate traffic):
```yaml
rate_limit:
  requests_per_second: 1
  burst_size: 10  # Increase from 5
```

2. Reduce message frequency:
   - Batch notifications instead of sending individually
   - Implement message queueing
   - Use thread replies instead of new messages

3. Check logs for retry attempts:
```bash
aofctl serve --gateway-config gateway.yaml --debug-gateway | grep "retry"
```

Gateway automatically retries with exponential backoff. The error logs show:
- Retry attempt number
- Delay before next retry
- Retry-After header value from platform

4. Platform-specific rate limits:
   - **Slack**: 1 req/sec (Tier 1), 20 req/min (Tier 2)
   - **Discord**: 10 req/sec per channel, 50 req/sec global
   - **Telegram**: 30 msg/sec to group, 1 msg/sec per user

### Gateway crashes on startup

**Symptom:** Gateway starts but crashes immediately

**Debug steps:**

1. Enable debug logging:
```bash
aofctl serve --gateway-config gateway.yaml --debug-gateway
```

2. Validate config:
```bash
aofctl serve --gateway-config gateway.yaml --validate-config
```

Expected output:
```
✓ Gateway config is valid
  Adapters: 2
  Squads: 1
```

3. Check adapter initialization logs:
```bash
aofctl serve --gateway-config gateway.yaml 2>&1 | grep "adapter"
```

Look for:
- `Registered gateway adapter: slack-Slack` - Success
- `Failed to create adapter for Slack: ...` - Failure with reason

4. Verify network connectivity to platform APIs:
```bash
# Slack
curl -I https://slack.com/api/auth.test

# Discord
curl -I https://discord.com/api/v10/users/@me

# Telegram
curl -I https://api.telegram.org/bot<token>/getMe
```

### Squad configuration errors

**Symptom:** Config validation fails with squad-related error

**Example errors:**
```
Duplicate squad name: 'ops-team'
Squad 'dev-team' must have at least one channel configured
Squad 'ops-team': Slack channel ID cannot be empty
```

**Solutions:**

1. **Duplicate squad names:**
```yaml
# ❌ Wrong
squads:
  - name: ops-team
  - name: ops-team  # Duplicate!

# ✅ Correct
squads:
  - name: ops-team
  - name: ops-team-2  # Unique name
```

2. **Missing channels:**
```yaml
# ❌ Wrong
squads:
  - name: dev-team
    agents: [agent1]
    channels: {}  # No channels!

# ✅ Correct
squads:
  - name: dev-team
    agents: [agent1]
    channels:
      slack: "C01234567"  # At least one channel
```

3. **Empty channel IDs:**
```yaml
# ❌ Wrong
channels:
  slack: ""  # Empty!

# ✅ Correct
channels:
  slack: "C01234567"
```

### Configuration parse errors

**Symptom:** Config loading fails with YAML parse error

**Example error:**
```
Config parse error at spec.adapters[0].config: invalid type: map, expected string
```

**Solutions:**

1. Check YAML syntax:
```bash
# Install yamllint
pip install yamllint

# Validate YAML
yamllint gateway.yaml
```

2. Verify JSON fields in adapter config:
```yaml
# ✅ Correct: JSON object for config
config:
  bot_token: "xoxb-..."
  app_token: "xapp-..."

# ❌ Wrong: String instead of object
config: "xoxb-..."
```

3. Check indentation (use 2 spaces, not tabs):
```yaml
# ✅ Correct
spec:
  runtime:
    websocket_url: "ws://..."

# ❌ Wrong (tabs)
spec:
	runtime:
		websocket_url: "ws://..."
```

4. Use serde_path_to_error output:

The error message shows exact field path:
```
Field: spec.squads[0].channels.slack
Error: invalid type: expected string, found null
```

This means: In first squad, Slack channel is null but should be string or omitted.

### WebSocket connection failures

**Symptom:** Gateway can't connect to agent runtime

**Example error:**
```
Failed to connect to WebSocket: Connection refused (ws://localhost:8080/ws)
```

**Solutions:**

1. Verify agent runtime is running:
```bash
# In separate terminal
aofctl serve --port 8080
```

2. Check WebSocket URL in config:
```yaml
runtime:
  websocket_url: "ws://localhost:8080/ws"  # Must match runtime port
```

3. Test WebSocket endpoint:
```bash
# Install websocat
brew install websocat

# Test connection
websocat ws://localhost:8080/ws
```

4. Check firewall rules:
```bash
# macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps

# Linux
sudo ufw status
```

## Debug Mode

Enable debug mode for verbose logging:

```bash
aofctl serve --gateway-config gateway.yaml --debug-gateway
```

Debug logs include:
- **Message content** (inbound/outbound)
- **API requests/responses** (headers, status codes)
- **Rate limiter stats** (tokens available, wait time)
- **Adapter lifecycle events** (start, stop, health checks)

Example debug output:
```
2026-02-13 10:23:45 DEBUG [aof_gateway::adapters::slack] Received message: channel=C01234567, user=U12345678, text="hello"
2026-02-13 10:23:45 DEBUG [aof_gateway::rate_limiter] Acquiring token: platform=Slack, available=4/5
2026-02-13 10:23:45 DEBUG [aof_gateway::adapters::slack] Sending message: channel=C01234567, text="Response"
2026-02-13 10:23:46 DEBUG [aof_gateway::rate_limiter] Token acquired: platform=Slack, wait_time=0ms
```

**Tip**: Pipe debug output to file for analysis:
```bash
aofctl serve --gateway-config gateway.yaml --debug-gateway 2>&1 | tee gateway-debug.log
```

## Performance Issues

### High latency

**Symptom:** Slow message delivery (>2 seconds)

**Diagnosis:**
1. Check rate limiter wait times (debug mode)
2. Verify network latency to platform APIs
3. Check CPU/memory usage

**Solutions:**
1. Increase rate limits if not hitting platform limits:
```yaml
rate_limit:
  requests_per_second: 5  # Increase from 1
```

2. Use thread replies instead of new messages (reduces API calls)

3. Batch notifications with squad broadcast

### Memory leaks

**Symptom:** Memory usage grows over time

**Diagnosis:**
```bash
# Monitor memory
top -pid $(pgrep aofctl)

# Or use htop
htop -p $(pgrep aofctl)
```

**Solutions:**
1. Restart gateway periodically (systemd timer, cron)
2. Check for unbounded message queues
3. Report issue with debug logs + memory profile

## Getting Help

### Collect diagnostic information

Before reporting issues, collect:

1. Gateway config (sanitized):
```bash
# Remove tokens before sharing
sed 's/bot_token: .*/bot_token: "REDACTED"/' gateway.yaml
```

2. Debug logs (last 50 lines):
```bash
aofctl serve --gateway-config gateway.yaml --debug-gateway 2>&1 | tail -50
```

3. Version information:
```bash
aofctl version
```

4. Platform details:
   - OS: macOS, Linux, Windows
   - Rust version: `rustc --version`
   - AOF version: `aofctl version`

### Support channels

- **GitHub issues**: https://github.com/agenticdevops/aof/issues
- **Documentation**: https://docs.aof.sh
- **Discord**: [Link to support channel]

### Reporting bugs

Include in bug report:
1. Minimal config that reproduces issue
2. Steps to reproduce
3. Expected vs actual behavior
4. Debug logs
5. Platform versions

**Good bug report:**
```markdown
**Environment:**
- OS: macOS 14.0
- AOF: v0.4.0-beta
- Platform: Slack

**Config:**
```yaml
# Minimal gateway.yaml (tokens redacted)
...
```

**Steps to reproduce:**
1. Start gateway: `aofctl serve --gateway-config gateway.yaml`
2. Send message in Slack: "hello"
3. Observe error

**Expected:** Bot responds with "Response"
**Actual:** Error: "Rate limit exceeded"

**Logs:**
```
2026-02-13 10:23:45 ERROR ...
```
```

## Common Patterns

### Multi-workspace Slack setup

```yaml
adapters:
  - platform: slack
    enabled: true
    config:
      bot_token: "${SLACK_WORKSPACE_1_TOKEN}"
      app_token: "${SLACK_WORKSPACE_1_APP_TOKEN}"
      bot_user_id: "U01234567"
      allowed_channels:
        - "C01234567"  # Limit to specific channels

  - platform: slack
    enabled: true
    config:
      bot_token: "${SLACK_WORKSPACE_2_TOKEN}"
      app_token: "${SLACK_WORKSPACE_2_APP_TOKEN}"
      bot_user_id: "U98765432"
```

### Development vs Production config

```yaml
# development.yaml
spec:
  adapters:
    - platform: slack
      enabled: true  # Only Slack for local testing
    - platform: discord
      enabled: false  # Disabled in development

# production.yaml
spec:
  adapters:
    - platform: slack
      enabled: true  # All platforms in production
    - platform: discord
      enabled: true
```

Switch between configs:
```bash
# Development
aofctl serve --gateway-config development.yaml

# Production
aofctl serve --gateway-config production.yaml
```

## See Also

- [Gateway Configuration Guide](../gateway-config.md)
- [Internal Architecture](../internal/03-messaging-gateway-architecture.md)
- [AOF Documentation](https://docs.aof.sh)
