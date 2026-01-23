# GitHub PR Review Setup - Quick Start

✅ **Configuration complete!** Your daemon config now includes GitHub integration.

## 1. Set Environment Variables

Add these to your `~/.zshrc` (or `~/.bashrc`):

```bash
# Required for GitHub
export GITHUB_TOKEN=ghp_your_github_token
export GITHUB_WEBHOOK_SECRET=$(openssl rand -hex 32)

# Required for LLM (using Gemini)
export GOOGLE_API_KEY=your_google_api_key

# Keep your existing Slack/Telegram (if using)
export SLACK_BOT_TOKEN=xoxb-...
export SLACK_SIGNING_SECRET=...
export TELEGRAM_BOT_TOKEN=...
```

### Get GitHub Token
1. Visit: https://github.com/settings/tokens
2. **Generate new token (classic)**
3. Select scope: `repo` (Full control)
4. Copy the token (starts with `ghp_`)

**IMPORTANT**: After setting environment variables, source your config:

```bash
source ~/.zshrc  # or source ~/.bashrc
```

## 2. Test the Setup

Before starting the server, verify everything works:

```bash
# Source environment variables
source ~/.zshrc

# Run the test script
./scripts/test-github-webhook.sh
```

You should see:
```
✓ Server is running
✓ GITHUB_WEBHOOK_SECRET is set
✓ GITHUB_TOKEN is set
✓ Ping event successful (HTTP 200)
✓ Pull request event successful (HTTP 200)
All tests passed! ✓
```

## 3. Start the Server

```bash
# Make sure to source environment variables first!
source ~/.zshrc

# From /Users/gshah/work/opsflow-sh/aof
./target/release/aofctl serve --config config/aof/daemon.yaml
```

**You should see:**
```
Starting AOF Trigger Server
  Bind address: 0.0.0.0:8080
  Registered platform: slack
  Registered platform: telegram
  Registered platform: github          ← This confirms it works!
Server starting...
```

## 3. Test the Setup

```bash
# Check if server is running
curl http://localhost:8080/health
# Should return: {"status":"ok"}

# Check platforms registered
curl http://localhost:8080/platforms
# Should include "github"
```

## 4. Configure GitHub Webhook (Local Testing)

### Start ngrok
```bash
ngrok http 8080
# Copy the HTTPS URL (e.g., https://abc123.ngrok.io)
```

### Add Webhook to GitHub
1. Go to your repository on GitHub
2. **Settings** → **Webhooks** → **Add webhook**
3. Configure:
   - **Payload URL**: `https://abc123.ngrok.io/webhook/github`
     - ⚠️ Important: `/webhook/github` (singular, not `/webhooks/`)
   - **Content type**: `application/json`
   - **Secret**: Your `GITHUB_WEBHOOK_SECRET` value
   - **Events**: Select "Pull requests"
   - **Active**: ✅ Checked
4. Click **Add webhook**

## 5. Test It!

### Create a Test PR
```bash
git checkout -b test-pr-review
echo "// Test" >> README.md
git add README.md
git commit -m "test: trigger PR review"
git push origin test-pr-review
```

Then open a PR on GitHub and watch:
- ✅ Your terminal logs (AOF server)
- ✅ ngrok dashboard (webhook received)
- ✅ GitHub PR comments (automated review)

## What's Configured

### Files Updated
```
config/aof/
  └── daemon.yaml          ← GitHub platform added, paths fixed

agents/
  ├── github-pr-reviewer.yaml  ← NEW PR review agent
  ├── devops-agent.yaml        (existing)
  ├── k8s-ops.yaml            (existing)
  └── sre-agent.yaml          (existing)

flows/
  └── (directory created for future use)
```

### Platforms Enabled
- ✅ Slack
- ✅ Telegram
- ✅ **GitHub** (NEW!)

### Changes Made
1. **Added GitHub platform** to `config/aof/daemon.yaml`
2. **Fixed directory paths** from `/app/agents` → `./agents`
3. **Created PR review agent** at `agents/github-pr-reviewer.yaml`
4. **Created flows directory** for future AgentFlow support

## Troubleshooting

### "GitHub platform not registered" or "GitHub enabled but missing webhook_secret"
**Solution:**
```bash
# Check if variables are set
echo $GITHUB_TOKEN
echo $GITHUB_WEBHOOK_SECRET

# If empty, source your config
source ~/.zshrc  # or source ~/.bashrc

# Verify they're set now
echo $GITHUB_WEBHOOK_SECRET

# Restart the server
./target/release/aofctl serve --config config/aof/daemon.yaml
```

### "Failed to load agents"
The paths are now relative (`./agents`), so make sure you run from the project root:
```bash
pwd
# Should be: /Users/gshah/work/opsflow-sh/aof
```

### "502 Bad Gateway" from ngrok
GitHub webhook URL must be `/webhook/github` (singular), not `/webhooks/github`.

### Agent not triggering
1. Check GitHub webhook deliveries (Settings → Webhooks → Recent Deliveries)
2. Look for green checkmark and HTTP 200 response
3. Check AOF server logs for "GitHub pull_request event"

## Next Steps

1. ✅ Environment variables set
2. ✅ Server running with GitHub registered
3. ✅ ngrok tunnel active
4. ✅ GitHub webhook configured
5. 🎯 Create a test PR and see the magic!

## Quick Reference

### Webhook Endpoint
```
http://localhost:8080/webhook/github  (local)
https://your-ngrok.ngrok.io/webhook/github  (ngrok)
```

### Server Command
```bash
./target/release/aofctl serve --config config/aof/daemon.yaml
```

### Test Webhook Locally
```bash
curl -X POST http://localhost:8080/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: ping" \
  -d '{"zen":"test"}'
```

---

**You're ready to go!** 🚀
