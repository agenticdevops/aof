# Mission Control: Conversational Agent Creation

Create agents, squads, skills, and schedules through natural language conversation.

## Getting Started

1. **Start AOF daemon:**
   ```bash
   aofctl serve
   ```

2. **Open Mission Control:**
   Navigate to http://localhost:8080 in your browser.

3. **Click "Create Agent":**
   In the top navigation, click the "+ Create Agent" button.

4. **Start chatting:**
   Describe what you want in natural language. The AI will guide you through the process.

---

## Creating Your First Agent

### Step 1: Open the conversation interface

Click "+ Create Agent" in Mission Control's navigation bar. You'll see a chat interface with example prompts.

### Step 2: Describe your agent

Type what you want:

```
I need a Kubernetes monitoring agent that checks pod health every 5 minutes
```

Press Enter to send.

### Step 3: Review generated files

The system will generate `AGENTS.md` and `SOUL.md` files for your agent. You'll see a preview showing:

- **workspace/AGENTS.md** - Agent configuration (ID, name, triggers, goals)
- **workspace/SOUL.md** - Agent personality and behavior guidelines

### Step 4: Confirm or edit

Three options:

- **Confirm & Save** - Accept the files and create the agent
- **Edit Before Saving** - Modify the YAML/Markdown before saving
- **Cancel** - Discard and start over

### Step 5: Agent appears in dashboard

After confirmation, navigate back to the Dashboard. Your new agent will appear in the agent grid.

---

## Building a Squad

A squad is a group of agents working together.

**Example conversation:**

```
User: I need an incident response squad with a triage agent, an investigator,
      and a communicator

AI: I'll create a 3-agent incident response squad for you.

    - **triage-agent:** Assesses severity and assigns priority
    - **incident-investigator:** Analyzes logs and identifies root cause
    - **incident-communicator:** Updates stakeholders and writes postmortems

    Does this match your needs?

User: Yes, create it

AI: [Generates squad configuration files]
```

**What gets created:**
- Three agents in `AGENTS.md`
- Three personality profiles in `SOUL.md`
- Squad coordination rules

---

## Teaching a Skill

Skills are reusable procedures that agents can execute.

**Example conversation:**

```
User: Teach agents how to restart a crashed service

AI: What steps should the agent follow to restart a service?

User: 1. Check if the service is running
      2. If not, run systemctl restart service-name
      3. Verify it started successfully
      4. Log the restart event

AI: I've created a "restart-service" skill with those steps.
```

**What gets created:**
- `skills/restart-service/SKILL.md` - Step-by-step instructions
- Skill reference added to relevant agents

---

## Setting Up Schedules

Configure agents to run on a schedule using natural language.

**Example conversation:**

```
User: Schedule the backup agent to run daily at 2am EST

AI: I'll create a schedule:
    - Agent: backup-agent
    - Frequency: Daily
    - Time: 2:00 AM EST (7:00 AM UTC)
    - Cron: 0 2 * * *

    Is this correct?

User: Yes

AI: Schedule created in triggers.yaml
```

**What gets created:**
- Entry in `triggers.yaml` with cron expression and timezone

---

## Editing Generated Files

If the AI doesn't generate exactly what you need:

1. **Click "Edit Before Saving"** in the file preview
2. A YAML/Markdown editor opens with line numbers
3. Make your changes
4. Click "Save Changes" to update the files
5. Click "Confirm & Save" to persist to workspace

**Editor features:**
- Line numbers for easy reference
- Syntax highlighting
- Unsaved changes indicator
- No changes? "Save Changes" button is disabled

---

## Power User Mode

For users comfortable with YAML/Markdown:

1. Start a conversation normally
2. When files are generated, click "Edit Before Saving"
3. Directly modify the YAML structure
4. Save and confirm

**Why edit manually?**
- Fine-tune agent goals
- Add custom metadata
- Specify exact tool configurations
- Override AI-generated personality traits

---

## Keyboard Shortcuts

- **Enter** - Send message
- **Shift+Enter** - New line in message (don't send)

---

## Tips for Better Results

### Be specific

Instead of:
```
Create a monitoring agent
```

Try:
```
Create a Kubernetes monitoring agent that checks pod health, CPU usage,
and memory consumption every 5 minutes, and alerts if any pod is unhealthy
```

### Provide context

```
I need an agent for our production Redis cluster that monitors replication lag
and automatically fails over to a replica if the primary becomes unavailable
```

### Iterate

If the first result isn't perfect, refine:

```
User: Add alerting to that agent
AI: What should trigger an alert?
User: Alert if replication lag exceeds 10 seconds
```

### Use examples

```
Create a deployment agent similar to the existing backup-agent, but for
deploying to staging environments
```

---

## What Each Conversation Type Creates

| Conversation Type | Files Created | Example |
|-------------------|---------------|---------|
| **Create Agent** | AGENTS.md, SOUL.md | "I need a K8s monitoring agent" |
| **Build Squad** | AGENTS.md (multiple), SOUL.md (multiple) | "Create an incident response squad" |
| **Teach Skill** | skills/{name}/SKILL.md | "Teach agents how to restart services" |
| **Configure Schedule** | triggers.yaml | "Run backup-agent daily at 2am" |

---

## Troubleshooting

**Chat doesn't respond:**
- Check WebSocket connection status in the top-right corner
- Ensure `aofctl serve` is running
- Reload the page

**Generated files don't match expectations:**
- Click "Edit Before Saving" to modify
- Or click "Cancel" and rephrase your request
- Provide more context in the conversation

**Agent doesn't appear in dashboard after creation:**
- Wait 5 seconds (config polling interval)
- Or reload the page manually

**"No pending files to confirm" error:**
- The AI needs more information
- Answer clarifying questions before files are generated

---

## Advanced Features

### Multi-turn conversations

The system remembers context:

```
User: Create a monitoring agent
AI: [Generates agent]
User: Add alerting to that agent
AI: [Updates the same agent with alerts]
```

### Clarifying questions

If your request is ambiguous, the AI will ask:

```
User: Create a deployment agent
AI: Which environment should this deploy to?
    - Production
    - Staging
    - Development
```

### File preview with tabs

When multiple files are generated, use tabs to switch between them:

```
[AGENTS.md] [SOUL.md] [SKILL.md]
```

---

## Next Steps

- **Learn more about agents:** See [Agent Configuration](./agents.md)
- **Understand personalities:** See [Agent Personas](./personas.md)
- **Create custom skills:** See [Skill Development](./skills.md)
- **Advanced scheduling:** See [Triggers & Schedules](./triggers.md)
