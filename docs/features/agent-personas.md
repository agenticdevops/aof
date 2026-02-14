# Agent Personas - Feature Reference

## Concepts

### Persona

A persona is the combination of identity, personality, and behavioral rules that define an agent's character. Personas make agents feel like team members with distinct voices, capabilities, and limitations.

A persona is composed from two workspace files:
- **AGENTS.md** -- Identity and capabilities (structured YAML)
- **SOUL.md** -- Personality and communication guidance (Markdown + YAML)

### System Prompt

A dynamically composed instruction set sent to the LLM before any user interaction. The persona system generates system prompts using 7-layer instruction layering, ensuring each agent responds in character.

### Introduction Event

A CoordinationEvent emitted when an agent starts up or joins a squad. Contains the agent's name, role, avatar, and introduction message. Visible in Mission Control and messaging gateways.

### Reliability Metrics

Computed values (uptime percentage, success rate) derived from an agent's event history. Displayed as color-coded badges in the UI to help operators calibrate trust.

## File Formats

### AGENTS.md

Defines the agent roster. YAML format with a top-level `agents` list.

```yaml
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    role: Infrastructure Specialist
    avatar: "\U0001F916"
    personality_traits:
      - methodical
      - detail-oriented
      - proactive
    can:
      - kubectl operations
      - pod debugging
      - log analysis
      - alerting
    cannot:
      - modify cluster RBAC (too dangerous)
      - delete persistent volumes without approval
    skills:
      - kubectl
      - pod-debugging
      - log-analysis
      - alerting
```

### SOUL.md

Defines personality guidance. Markdown with YAML code blocks per agent.

See the [tutorial](../tutorials/create-agent-persona.md) for the complete format specification.

## HTTP APIs

### GET /api/config/agents

Returns the full agent roster with persona fields.

**Request:**
```
GET /api/config/agents HTTP/1.1
Host: localhost:3030
```

**Response:**
```json
{
  "agents": [
    {
      "id": "k8s-monitor",
      "name": "Kubernetes Monitor",
      "role": "Infrastructure Specialist",
      "avatar": "\ud83e\udd16",
      "personality_traits": ["methodical", "detail-oriented", "proactive"],
      "status": "idle",
      "can": [
        "kubectl operations",
        "pod debugging",
        "log analysis",
        "alerting"
      ],
      "cannot": [
        "modify cluster RBAC (too dangerous)",
        "delete persistent volumes without approval"
      ],
      "skills": ["kubectl", "pod-debugging", "log-analysis", "alerting"]
    }
  ],
  "tools": [
    {
      "name": "kubectl",
      "description": "Kubernetes CLI for cluster management",
      "category": "infrastructure"
    }
  ]
}
```

**Headers:**
- `X-Config-Version: sha256_hash` -- SHA256 hash of concatenated AGENTS.md + TOOLS.md content. Changes when files are modified. Use for client-side cache invalidation.

**Status codes:**
- `200 OK` -- Success
- `500 Internal Server Error` -- Workspace files could not be loaded

### GET /api/agents/:id/metrics

Returns reliability metrics for a specific agent.

**Request:**
```
GET /api/agents/k8s-monitor/metrics HTTP/1.1
Host: localhost:3030
```

**Response (sufficient data):**
```json
{
  "agent_id": "k8s-monitor",
  "uptime_percent": 95.5,
  "success_rate": 92.0,
  "event_count": 200,
  "last_update": "2026-02-14T10:30:00Z",
  "last_error": "2026-02-14T09:15:22Z"
}
```

**Response (insufficient data, < 10 events):**
```json
{
  "agent_id": "k8s-monitor",
  "uptime_percent": null,
  "success_rate": null,
  "event_count": 5,
  "last_update": "2026-02-14T10:30:00Z",
  "last_error": null
}
```

**Headers:**
- `X-Metrics-Version: 42` -- Monotonically increasing version counter. Changes on every event update. Use for polling optimization.

**Status codes:**
- `200 OK` -- Metrics returned (may have null percentages if insufficient data)
- `404 Not Found` -- Agent has no recorded events

### Metric Computation Rules

- **Uptime %** = (total events - error events) / total events * 100
- **Success rate** = completed events / total events * 100
- **Minimum events:** 10 events required before percentages are computed. Below this threshold, `uptime_percent` and `success_rate` return `null`.
- **Cache capacity:** 10,000 events maximum (FIFO eviction for older events)

## WebSocket Events

### Connection

```
ws://localhost:3030/ws
```

All CoordinationEvents are broadcast to WebSocket subscribers in real time.

### AgentIntroduction Event

Emitted at daemon startup for each configured agent.

```json
{
  "activity": {
    "activity_type": "Info",
    "message": "Agent introduction: Kubernetes Monitor",
    "metadata": {}
  },
  "agent_id": "k8s-monitor",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "timestamp": "2026-02-14T10:00:00Z",
  "introduction": {
    "agent_id": "k8s-monitor",
    "agent_name": "Kubernetes Monitor",
    "role": "Infrastructure Specialist",
    "avatar": "\ud83e\udd16",
    "intro_message": "I'm Kubernetes Monitor, your infrastructure specialist. I watch your clusters constantly and raise the alarm when something needs attention.",
    "personality_summary": "A methodical Kubernetes specialist who takes system health seriously. Prefers data-driven decisions and reports issues before they become incidents.",
    "skills": ["kubectl", "pod-debugging", "log-analysis", "alerting"]
  }
}
```

**Note:** The `introduction` field is only present on introduction events. Regular activity events omit it (via `skip_serializing_if`).

### Activity Events (Regular)

Standard agent activity events do not include introduction data:

```json
{
  "activity": {
    "activity_type": "Completed",
    "message": "Checked pod health: all 12 pods healthy",
    "metadata": {}
  },
  "agent_id": "k8s-monitor",
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "timestamp": "2026-02-14T10:05:00Z"
}
```

## Integration Points

### Where Personas Appear

| Location | What's Shown | Source |
|----------|-------------|--------|
| System Prompt | Full personality + role + capabilities | AGENTS.md + SOUL.md via PromptComposer |
| AgentCard (UI) | Avatar, name, role, traits, metrics | /api/config/agents + /api/agents/:id/metrics |
| Introduction Toast (UI) | Name, role, intro message | WebSocket AgentIntroduction event |
| Event Stream (UI) | Agent name + activity | WebSocket CoordinationEvent |
| Messaging Gateway | Agent responds in character | System prompt composition |

### Reliability Badges

Color coding in the UI:

| Metric Value | Color | Meaning |
|-------------|-------|---------|
| >= 95% | Green | Excellent reliability |
| >= 80% | Yellow/Amber | Acceptable, watch closely |
| < 80% | Red | Needs attention |
| null (< 10 events) | Gray ("--") | Insufficient data |

## Configuration

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `AOF_WORKSPACE` | `workspace/` | Path to workspace directory containing AGENTS.md and SOUL.md |

### Command Line

```bash
# Start daemon with custom workspace
aofctl serve --workspace /path/to/workspace/

# Start with debug logging (see prompt composition)
RUST_LOG=debug aofctl serve
```

### Optional: squads.yaml

Override introduction messages per squad:

```yaml
squads:
  incident-response:
    name: Incident Response Team
    agents:
      - id: incident-responder
        intro_override: "Ready for incident response. I'm coordinating the team."
      - id: log-analyzer
        intro_override: "I'll dig into the logs for this incident."
```

Place in the workspace directory. The daemon reads it on startup if present. If missing or invalid, default introductions from SOUL.md are used.

## FAQ

### Why is my agent not using its persona?

1. **Check SOUL.md exists:** The daemon logs a warning if SOUL.md is missing. Without it, agents use default personality.
2. **Check the daemon logs:** Look for `"Prompt cache miss for agent 'your-agent-id'"` to confirm the prompt was composed.
3. **Check SOUL.md format:** The YAML block must be inside triple-backtick `yaml` fences. Missing fences cause the section to be skipped.
4. **Verify the ID matches:** The `id` in SOUL.md must exactly match the `id` in AGENTS.md (case-sensitive).

### Why is the metric showing null?

Metrics require at least 10 events before computing percentages. Check:
1. Has the agent run at least 10 tasks? Check with: `curl localhost:3030/api/agents/your-agent-id/metrics | jq .event_count`
2. If event_count < 10, the UI shows "--" instead of a percentage. This is intentional to avoid misleading statistics.

### Avatar emoji displaying wrong?

1. Some emoji render differently across browsers and operating systems.
2. Use common emoji from the Emoticons block (U+1F600-1F64F) for best compatibility.
3. Recommended safe choices: robot, magnifying glass, alarm, shield, gear, wrench, elephant, snake, bear.
4. Avoid ZWJ sequences (emoji composed of multiple characters joined by zero-width joiners) as they may render as multiple characters on some platforms.

### Introduction message not appearing?

1. **Check SOUL.md:** Ensure the `default_intro` field is non-empty.
2. **Check WebSocket:** Connect with `websocat ws://localhost:3030/ws` and restart the daemon. You should see introduction events.
3. **Check the UI:** Introduction toasts display for 8 seconds and queue if more than 3 arrive simultaneously.
4. **Check daemon logs:** Look for `"Emitting introduction events"` on startup.

### Skill not found in TOOLS.md?

Agent skills reference tool names in TOOLS.md. If a skill doesn't match:
1. Check that tool names are lowercase-hyphenated in both files.
2. Add the missing tool to TOOLS.md, or update the agent's skills list.
3. The daemon logs a warning: `"Skill 'unknown-tool' not found in TOOLS.md for agent 'your-agent'"`.
4. The prompt will still compose but will note the tool as "not found in TOOLS.md".

### Prompt too long, truncation occurred?

If the daemon logs `"Persona prompt truncation needed"`:
1. **Reduce skills:** Agents with 50+ skills produce very long tool sections.
2. **Shorten communication guide:** Keep the prose focused and concise.
3. **Split into specialized agents:** Instead of one agent with 50 skills, create 2-3 focused agents.
4. The truncation is graceful: personality and role are never removed. Behavioral rules and tool descriptions are dropped first.
