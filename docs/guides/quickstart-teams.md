# Teams Bot Quickstart

Get AOF running with Microsoft Teams in 15 minutes.

## Prerequisites

- Azure account ([portal.azure.com](https://portal.azure.com))
- Microsoft 365 tenant with Teams
- AOF installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- Public HTTPS endpoint (or ngrok for development)

## Step 1: Create Azure Bot Service

### Option A: Azure Portal (Recommended)

1. Go to [Azure Portal](https://portal.azure.com)
2. Search for "Azure Bot" and click Create
3. Fill in:
   - **Bot handle**: `aof-ops-bot`
   - **Subscription**: Your subscription
   - **Resource group**: Create new or use existing
   - **Pricing tier**: F0 (Free) for testing
   - **Microsoft App ID**: Create new

4. Click "Create" and wait for deployment

### Option B: Azure CLI

```bash
# Login to Azure
az login

# Create resource group
az group create --name aof-bot-rg --location eastus

# Create bot
az bot create \
  --resource-group aof-bot-rg \
  --name aof-ops-bot \
  --kind registration \
  --sku F0
```

## Step 2: Get Bot Credentials

1. Open your bot in Azure Portal
2. Go to **Configuration**
3. Note the **Microsoft App ID**
4. Click **Manage Password** next to App ID
5. Create a new client secret and save it

```bash
# Set environment variables
export TEAMS_APP_ID="your-microsoft-app-id"
export TEAMS_APP_PASSWORD="your-client-secret"
```

## Step 3: Configure Messaging Endpoint

### For Development (ngrok)

```bash
# Start ngrok
ngrok http 8080

# Note the HTTPS URL: https://xxx.ngrok.io
```

### For Production

Use your public domain with HTTPS.

### Set Endpoint in Azure

1. In Azure Portal, go to your bot's **Configuration**
2. Set **Messaging endpoint**: `https://your-domain.com/webhook/teams`
3. Save changes

## Step 4: Enable Teams Channel

1. In Azure Portal, go to your bot
2. Click **Channels**
3. Click **Microsoft Teams** icon
4. Accept terms and click **Apply**
5. Teams channel is now enabled

## Step 5: Create AOF Configuration

```bash
mkdir -p ~/.aof
```

### Trigger Configuration

```yaml
# ~/.aof/triggers/teams-starter.yaml
apiVersion: aof.dev/v1
kind: Trigger
metadata:
  name: teams-starter
  labels:
    platform: teams
spec:
  type: Teams
  config:
    app_id: ${TEAMS_APP_ID}
    app_password: ${TEAMS_APP_PASSWORD}

    # Optional: Restrict to your tenant
    # allowed_tenants:
    #   - "your-tenant-id"

  commands:
    /help:
      agent: devops
      description: "Show available commands"

    /status:
      agent: devops
      description: "Check system status"

    /pods:
      agent: k8s-ops
      description: "List Kubernetes pods"

  default_agent: devops
```

### Agent Configuration

```yaml
# ~/.aof/agents/devops.yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: devops
spec:
  model: google:gemini-2.5-flash
  temperature: 0
  max_tokens: 2048

  description: "DevOps assistant for Teams"

  tools:
    - kubectl
    - docker
    - helm

  system_prompt: |
    You are a DevOps assistant in Microsoft Teams.

    ## Response Guidelines
    - Be concise but informative
    - Use Adaptive Card formatting when helpful
    - Offer relevant follow-up actions
    - Include status indicators: ✅ ⚠️ ❌

    ## For status queries, respond like:
    📊 **System Status**

    • API: ✅ Healthy
    • Database: ✅ Connected
    • Cache: ⚠️ High memory

    Would you like more details on any service?
```

### Daemon Configuration

```yaml
# ~/.aof/daemon.yaml
log_level: info
http_port: 8080

platforms:
  teams:
    enabled: true
    webhook_path: /webhook/teams
```

## Step 6: Start AOF

```bash
# Validate configuration
aofctl validate ~/.aof/triggers/teams-starter.yaml
aofctl validate ~/.aof/agents/devops.yaml

# Start daemon
aofctl daemon start

# Check status
aofctl daemon status
```

## Step 7: Install Bot in Teams

### Method A: Teams Admin Center

1. Go to [Teams Admin Center](https://admin.teams.microsoft.com)
2. Navigate to **Teams apps** > **Manage apps**
3. Click **Upload new app**
4. Upload your bot manifest (see below)

### Method B: Developer Portal

1. Go to [Teams Developer Portal](https://dev.teams.microsoft.com)
2. Click **Apps** > **Import app**
3. Create manifest with your bot ID

### Minimal App Manifest

Create `manifest.json`:

```json
{
  "$schema": "https://developer.microsoft.com/json-schemas/teams/v1.16/MicrosoftTeams.schema.json",
  "manifestVersion": "1.16",
  "version": "1.0.0",
  "id": "your-app-id-here",
  "packageName": "com.yourcompany.aofbot",
  "developer": {
    "name": "Your Company",
    "websiteUrl": "https://yourcompany.com",
    "privacyUrl": "https://yourcompany.com/privacy",
    "termsOfUseUrl": "https://yourcompany.com/terms"
  },
  "name": {
    "short": "OpsBot",
    "full": "AOF Operations Bot"
  },
  "description": {
    "short": "DevOps assistant",
    "full": "AI-powered DevOps assistant for infrastructure management"
  },
  "icons": {
    "color": "color.png",
    "outline": "outline.png"
  },
  "accentColor": "#0078D4",
  "bots": [
    {
      "botId": "your-microsoft-app-id",
      "scopes": ["personal", "team", "groupchat"],
      "supportsFiles": false,
      "isNotificationOnly": false,
      "commandLists": [
        {
          "scopes": ["personal", "team", "groupchat"],
          "commands": [
            { "title": "help", "description": "Show available commands" },
            { "title": "status", "description": "Check system status" },
            { "title": "pods", "description": "List Kubernetes pods" }
          ]
        }
      ]
    }
  ],
  "permissions": ["identity", "messageTeamMembers"],
  "validDomains": ["your-domain.com"]
}
```

Zip `manifest.json` with icon files and upload.

## Step 8: Test Your Bot

### In Teams

1. Open Teams
2. Find your bot in Apps or search for "OpsBot"
3. Start a chat
4. Try these commands:

```
/help
/status
check the pods in default namespace
```

### Expected Response

```
📊 System Status

✅ All systems operational

• API Gateway: Running
• Database: Connected
• Cache: Healthy

[View Details] [Check Logs]
```

## Troubleshooting

### Bot Not Responding

```bash
# Check daemon is running
aofctl daemon status

# Check logs
aofctl daemon logs --follow

# Verify webhook is accessible
curl -X POST https://your-domain.com/webhook/teams \
  -H "Content-Type: application/json" \
  -d '{"type": "ping"}'
```

### Authentication Errors

```bash
# Verify credentials
echo $TEAMS_APP_ID
echo $TEAMS_APP_PASSWORD

# Test token generation
curl -X POST https://login.microsoftonline.com/botframework.com/oauth2/v2.0/token \
  -d "grant_type=client_credentials&client_id=$TEAMS_APP_ID&client_secret=$TEAMS_APP_PASSWORD&scope=https://api.botframework.com/.default"
```

### Ngrok Issues

```bash
# Restart ngrok with new tunnel
ngrok http 8080

# Update messaging endpoint in Azure Portal
# Don't forget to update after each ngrok restart
```

## Next Steps

- [Teams Tutorial](../tutorials/teams-ops-bot.md) - Build a complete ops bot
- [Teams Reference](../reference/teams-integration.md) - Full API reference
- [Approval Workflow](./approval-workflow.md) - Add deployment approvals
- [Custom Tools](../tools/custom-tools.md) - Add your own tools
