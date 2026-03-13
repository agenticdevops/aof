# Gateway HTTP API

**Version:** v1 (2026-03-12)
**Module:** `agentix-runtime::gateway`

---

## Overview

The OpenAgentiX gateway is an HTTP server that loads agent directories, manages runs, and streams ReAct loop output to HTTP clients via Server-Sent Events (SSE).

Started with `agentix gateway start` (CLI-10) or `Gateway::start()` programmatically.

---

## Base URL

```
http://<host>:<port>   (default: http://127.0.0.1:7777)
```

---

## Authentication

None by default. Production deployments should put the gateway behind a reverse proxy with authentication.

---

## Endpoints

### Health

#### `GET /healthz`

Returns 200 OK when the gateway is running.

**Response:**
```json
{
  "status": "ok",
  "version": "2.0.0-alpha"
}
```

---

### Agents

#### `GET /api/v1/agents`

List all loaded agents.

**Response:** Array of agent summaries
```json
[
  {
    "name": "my-agent",
    "description": "A helpful agent",
    "model": "anthropic/claude-sonnet-4-6",
    "status": "ready",
    "loaded_at": "2026-03-12T00:00:00Z"
  }
]
```

**Status values:** `ready` | `running` | `error`

---

#### `POST /api/v1/agents`

Register a new agent from inline YAML.

**Request body:**
```json
{
  "yaml": "<flat YAML agent spec string>"
}
```

**Example YAML:**
```yaml
apiVersion: openagentix.dev/v1
kind: Agent
metadata:
  name: my-agent
spec:
  model: anthropic/claude-sonnet-4-6
  system_prompt: "You are a helpful assistant."
```

**Responses:**
- `201 Created` — Agent registered: `{"name":"my-agent","status":"ready"}`
- `400 Bad Request` — Invalid YAML: `{"error":"...","field":"spec.model"}`
- `409 Conflict` — Agent already exists: `{"error":"Agent 'my-agent' already exists. Use PUT to update."}`

---

#### `GET /api/v1/agents/:name`

Get details for a specific agent.

**Responses:**
- `200 OK` — Agent summary (same shape as list item)
- `404 Not Found` — `{"error":"Agent 'name' not found"}`

---

#### `PUT /api/v1/agents/:name`

Update an existing agent from inline YAML.

**Request body:** Same as POST

**Responses:**
- `200 OK` — Agent updated: `{"name":"my-agent","status":"updated"}`
- `400 Bad Request` — Invalid YAML
- `404 Not Found` — Agent doesn't exist (use POST to create)

---

### Running Agents

#### `POST /api/v1/agents/:name/run`

Start an agent run. Returns a Server-Sent Events stream of `ReActEvent`s.

**Request body:**
```json
{
  "input": "Your task or question"
}
```

**Query parameters:**
- `?format=json` — Return NDJSON stream instead of SSE (for non-browser clients)

**SSE Response (default):**
```
Content-Type: text/event-stream

event: react_step
data: {"phase":"step","plan":"I'll check the current status...","action":{"tool_name":"kubectl","input":{"command":"get pods"}},"observation":"NAME   READY   STATUS..."}

event: complete
data: {"output":"The pods are all running.","iterations":2,"reached_max_iterations":false}
```

**Event types:**
| Event name | Description |
|-----------|-------------|
| `react_step` | One plan-act-observe cycle completed |
| `complete` | Run finished successfully |
| `error` | Run failed with an error |

**Responses:**
- `200 OK` — SSE stream
- `404 Not Found` — Agent not found

---

#### `GET /api/v1/agents/:name/runs`

List all runs for an agent, most recent first.

**Response:** Array of run summaries
```json
[
  {
    "id": "uuid",
    "agent_name": "my-agent",
    "started_at": "2026-03-12T00:00:00Z",
    "completed_at": "2026-03-12T00:01:00Z",
    "status": "completed",
    "input": "What pods are running?",
    "output": "The following pods are running...",
    "iterations": 2
  }
]
```

**Run status values:** `running` | `completed` | `failed` | `cancelled`

---

#### `GET /api/v1/agents/:name/runs/:run_id`

Get details of a specific run.

**Responses:**
- `200 OK` — Run summary
- `404 Not Found` — Run not found

---

#### `GET /api/v1/agents/:name/runs/:run_id/logs`

Get the output log for a run.

**Response:**
```json
{
  "run_id": "uuid",
  "output": "Final agent output...",
  "iterations": 3,
  "status": "completed"
}
```

---

#### `DELETE /api/v1/agents/:name/runs/:run_id`

Stop a running agent by sending a cancellation signal.

**Responses:**
- `200 OK` — `{"message":"Run stopped"}`
- `404 Not Found` — Run not found or already completed

---

## Error Response Format

All error responses use the same shape:

```json
{
  "error": "Human-readable description of what went wrong",
  "field": "optional.yaml.field.path"
}
```

The `field` property is omitted when not applicable.

---

## Agent Loading

On startup, the gateway scans `agents_dir` (default: `./agents`) and loads all agents:

- **Directory entries** with `agent.yaml` → loaded as GitAgent directory format
- **`*.yaml` / `*.yml` files** → loaded as flat YAML backward-compat format

Failed agents are logged as warnings; other agents continue loading.

---

---

## WebSocket — Real-Time Event Stream

### `GET /ws`

Upgrade to a WebSocket connection and receive real-time typed JSON events from the gateway.

The Command Center uses this endpoint instead of polling the REST API. Any client that needs live updates (agent status changes, run completions, approval requests) should connect here.

**Protocol:** HTTP → WebSocket upgrade (standard `Upgrade: websocket` handshake).

**Connection flow:**

1. Client connects and sends `Upgrade: websocket` headers.
2. Server upgrades the connection and immediately sends a `Connected` event.
3. Server broadcasts all subsequent events to every connected client.
4. Client receives events as JSON text frames.
5. Client may close the connection at any time; server handles cleanup gracefully.

**Event envelope:** Every event is a JSON object with a `"type"` discriminant.

```json
{ "type": "connected", "message": "Connected to OpenAgentiX gateway" }
```

**Event reference:**

| `type`                | Fields                                                       | Triggered by                           |
|-----------------------|--------------------------------------------------------------|----------------------------------------|
| `connected`           | `message: string`                                            | On every new WebSocket connection      |
| `agent_status`        | `agent_name: string`, `status: string`                       | Agent status changes                   |
| `run_started`         | `agent_name: string`, `run_id: string`                       | `POST /api/v1/agents/:name/run`        |
| `run_completed`       | `agent_name`, `run_id`, `status`, `duration_ms?: number`     | Run lifecycle events                   |
| `approval_requested`  | `id: string`, `agent_name: string`, `action: string`         | Approval gate in agent execution       |
| `approval_decided`    | `id: string`, `decision: "approved" \| "denied"`             | `POST /approvals/:id/approve` or deny  |
| `cost_update`         | `agent_name: string`, `total_cost_usd: number`               | After each run (LLM cost update)       |

**Example events:**

```json
{"type":"connected","message":"Connected to OpenAgentiX gateway"}
{"type":"run_started","agent_name":"deploy-agent","run_id":"a1b2-c3d4"}
{"type":"approval_requested","id":"req-001","agent_name":"deploy-agent","action":"kubectl apply -f deployment.yaml"}
{"type":"approval_decided","id":"req-001","decision":"approved"}
{"type":"run_completed","agent_name":"deploy-agent","run_id":"a1b2-c3d4","status":"success","duration_ms":4200}
{"type":"cost_update","agent_name":"deploy-agent","total_cost_usd":0.0043}
```

**Fan-out:** All connected clients receive the same events. The channel has capacity 256; slow clients are dropped (lagged) rather than blocking event producers.

**JavaScript client example:**

```javascript
const ws = new WebSocket('ws://localhost:7777/ws');

ws.onmessage = ({ data }) => {
  const event = JSON.parse(data);
  switch (event.type) {
    case 'run_started':
      console.log(`Run ${event.run_id} started for ${event.agent_name}`);
      break;
    case 'approval_requested':
      promptUser(event.id, event.action);
      break;
    case 'run_completed':
      console.log(`Run done in ${event.duration_ms}ms: ${event.status}`);
      break;
  }
};
```

---

## CORS

All endpoints (including `/ws` upgrade) have permissive CORS headers enabled (`CorsLayer::permissive()`). For production, configure your reverse proxy to restrict origins.

---

## Starting the Gateway

```bash
# Start with workspace config (reads agentix.yaml)
agentix gateway start

# Start with explicit port
agentix gateway start --port 8080

# Start pointing to agents directory
agentix gateway start --agents-dir ./my-agents/
```

See `workspace-config.md` for gateway configuration options.
