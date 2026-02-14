# AOF Complete API Specification

**Version:** 1.0
**Status:** Active (Phase 7+)
**Last Updated:** 2026-02-14
**Audience:** Frontend Developers, API Integrators, builder.io

---

## Overview

This specification defines **all** REST API endpoints, WebSocket messages, and integration contracts for the AOF Mission Control web application. Use this document to:

- Build frontend components that consume these endpoints
- Create external integrations using AOF APIs
- Configure builder.io to construct the complete web UI
- Understand real-time event streaming via WebSocket

---

## Table of Contents

1. [Base URLs](#base-urls)
2. [Core Endpoints](#core-endpoints)
3. [Webhook Endpoints](#webhook-endpoints)
4. [Configuration Endpoints](#configuration-endpoints)
5. [Metrics Endpoints](#metrics-endpoints)
6. [Coordination Endpoints](#coordination-endpoints)
7. [Conversation Endpoints](#conversation-endpoints)
8. [WebSocket Events](#websocket-events)
9. [Error Handling](#error-handling)
10. [Authentication](#authentication)

---

## Base URLs

| Environment | URL | Port | Protocol |
|-------------|-----|------|----------|
| Local Development | `http://localhost` | 7777 | HTTP/WS |
| Docker | `http://aof-daemon` | 7777 | HTTP/WS |
| Production | `https://aof-api.example.com` | 443 | HTTPS/WSS |

---

## Core Endpoints

### Health Check

#### `GET /health`

Server liveness and readiness check.

**Request:**
```bash
curl -X GET http://localhost:7777/health
```

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2026-02-14T09:30:15.123Z"
}
```

**Status Codes:**
- `200 OK` - Server is healthy
- `503 Service Unavailable` - Server is not ready

---

## Webhook Endpoints

### Receive Platform Messages

#### `POST /webhook/:platform`

Receive webhooks from messaging platforms. AOF daemon accepts events and processes them asynchronously.

**Platforms Supported:**
- `slack` - Slack events and slash commands
- `discord` - Discord messages and interactions
- `telegram` - Telegram messages
- `whatsapp` - WhatsApp messages
- `github` - GitHub push, PR, issue events
- `jira` - Jira issue events

**Request:**
```bash
# Slack example
curl -X POST http://localhost:7777/webhook/slack \
  -H "Content-Type: application/json" \
  -H "X-Slack-Request-Timestamp: 1614000000" \
  -H "X-Slack-Signature: v0=..." \
  -d '{"type": "event_callback", "event": {"type": "message", "text": "@aofbot status"}}'
```

**Response (202 Accepted):**
```json
{
  "status": "accepted"
}
```

**Status Codes:**
- `200 OK` - Webhook accepted (fire-and-forget processing)
- `202 Accepted` - Processing queued
- `400 Bad Request` - Invalid webhook format
- `404 Not Found` - Unknown platform
- `401 Unauthorized` - Invalid signature

**Webhook Signatures:**
Each platform requires verification:
- **Slack**: `X-Slack-Signature` HMAC-SHA256
- **Discord**: `X-Signature-Ed25519` + `X-Signature-Timestamp`
- **GitHub**: `X-Hub-Signature-256` HMAC-SHA256
- **Telegram**: Webhook secret verification
- **WhatsApp**: HMAC verification

---

## Configuration Endpoints

### List Available Agents

#### `GET /api/config/agents`

Fetch all registered agents (from discovery or config).

**Request:**
```bash
curl -X GET http://localhost:7777/api/config/agents
```

**Response (200 OK):**
```json
{
  "agents": [
    {
      "id": "kubo",
      "name": "Kubernetes Expert",
      "description": "Kubernetes cluster administration",
      "model": "google:gemini-2.5-flash",
      "capabilities": ["k8s", "containers", "orchestration"],
      "config_path": "quickstart/agents/kubo.yaml"
    },
    {
      "id": "doku",
      "name": "Docker Specialist",
      "description": "Container best practices",
      "model": "google:gemini-2.5-flash",
      "capabilities": ["docker", "containers", "images"],
      "config_path": "quickstart/agents/doku.yaml"
    }
  ],
  "total": 2,
  "discovered_at": "2026-02-14T09:00:00.000Z"
}
```

**Response Schema:**
```typescript
interface AgentConfig {
  id: string;
  name: string;
  description: string;
  model: string;
  capabilities: string[];
  config_path: string;
}

interface AgentsResponse {
  agents: AgentConfig[];
  total: number;
  discovered_at: string;  // ISO 8601 timestamp
}
```

**Status Codes:**
- `200 OK` - Agent list returned
- `204 No Content` - No agents discovered
- `503 Service Unavailable` - Agent discovery failed

---

### List Available Tools

#### `GET /api/config/tools`

Fetch all tools available to agents (MCP servers, local tools, etc).

**Request:**
```bash
curl -X GET http://localhost:7777/api/config/tools
```

**Response (200 OK):**
```json
{
  "tools": [
    {
      "id": "kubernetes",
      "name": "Kubernetes CLI",
      "description": "Direct kubectl access",
      "type": "local",
      "provider": "kubectl"
    },
    {
      "id": "docker",
      "name": "Docker Daemon",
      "description": "Docker container operations",
      "type": "local",
      "provider": "docker"
    }
  ],
  "total": 2
}
```

**Status Codes:**
- `200 OK` - Tool list returned
- `204 No Content` - No tools available

---

### Get Configuration Version

#### `GET /api/config/version`

Check configuration freshness (for cache validation).

**Response (200 OK):**
```json
{
  "config_version": "20260214-093015",
  "agents_count": 11,
  "tools_count": 5,
  "loaded_at": "2026-02-14T09:00:00.000Z",
  "workspace": "/Users/gshah/work/opsflow-sh/aof"
}
```

---

## Metrics Endpoints

### Get Agent Metrics

#### `GET /api/agents/:id/metrics`

Fetch performance and reliability metrics for a specific agent.

**Request:**
```bash
curl -X GET http://localhost:7777/api/agents/kubo/metrics
```

**Response (200 OK):**
```json
{
  "agent_id": "kubo",
  "reliability": {
    "success_rate": 0.95,
    "average_response_ms": 2341,
    "total_invocations": 156,
    "failures": 8,
    "last_failure": "2026-02-14T08:30:00.000Z"
  },
  "tokens": {
    "total_spent": 45230,
    "average_per_call": 289,
    "peak_call": 1205
  },
  "recent_calls": [
    {
      "timestamp": "2026-02-14T09:25:00.000Z",
      "status": "success",
      "tokens": 312,
      "duration_ms": 2105
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Metrics available
- `404 Not Found` - Agent not found
- `204 No Content` - No metrics yet

---

## Coordination Endpoints

### Get Coordination Health

#### `GET /api/coordination/health`

Fetch current agent health status and heartbeat information.

**Request:**
```bash
curl -X GET http://localhost:7777/api/coordination/health
```

**Response (200 OK):**
```json
{
  "agents": [
    {
      "agent_id": "k8s-monitor",
      "status": "Healthy",
      "last_heartbeat": "2026-02-14T09:30:15.123Z",
      "consecutive_misses": 0,
      "last_response_ms": 145,
      "degraded_reason": null
    },
    {
      "agent_id": "network-watch",
      "status": "Unresponsive",
      "last_heartbeat": "2026-02-14T09:28:00.000Z",
      "consecutive_misses": 3,
      "last_response_ms": null,
      "degraded_reason": "Heartbeat timeout"
    }
  ],
  "heartbeat_config": {
    "frequency_secs": 60,
    "timeout_secs": 120
  },
  "coordination_enabled": true
}
```

**Status Codes:**
- `200 OK` - Health data returned
- `503 Service Unavailable` - Coordination disabled

---

### Get Coordination Metrics

#### `GET /api/coordination/metrics`

Fetch coordination token overhead and operational mode status.

**Request:**
```bash
curl -X GET http://localhost:7777/api/coordination/metrics
```

**Response (200 OK):**
```json
{
  "coordination_tokens": 4200,
  "production_tokens": 95800,
  "overhead_percent": 4.2,
  "heartbeat_tokens": 2100,
  "standup_tokens": 2100,
  "current_mode": "Full",
  "auto_degrade_enabled": true,
  "max_overhead_percent": 30,
  "window_start": "2026-02-14T09:00:00.000Z"
}
```

**Status Codes:**
- `200 OK` - Metrics available
- `503 Service Unavailable` - Coordination disabled

---

### Set Coordination Mode

#### `POST /api/coordination/mode`

Manually override coordination mode (for testing or emergency control).

**Request:**
```bash
curl -X POST http://localhost:7777/api/coordination/mode \
  -H "Content-Type: application/json" \
  -d '{"mode": "Reduced", "reason": "User manual override"}'
```

**Request Body:**
```typescript
interface SetModeRequest {
  mode: "Full" | "Standard" | "Reduced" | "HeartbeatOnly" | "Disabled";
  reason?: string;
}
```

**Response (200 OK):**
```json
{
  "previous_mode": "Full",
  "new_mode": "Reduced",
  "auto_degrade_enabled": false,
  "reason": "User manual override"
}
```

**Status Codes:**
- `200 OK` - Mode changed
- `400 Bad Request` - Invalid mode
- `503 Service Unavailable` - Coordination disabled

---

### Trigger Manual Standup

#### `POST /api/coordination/standup/trigger`

Manually trigger an immediate agent status check (standup).

**Request:**
```bash
curl -X POST http://localhost:7777/api/coordination/standup/trigger \
  -H "Content-Type: application/json" \
  -d '{"reason": "Incident response check-in"}'
```

**Response (202 Accepted):**
```json
{
  "request_id": "standup-2026-02-14-0930",
  "status": "processing",
  "expected_completion_ms": 5000
}
```

**Status Codes:**
- `202 Accepted` - Standup triggered
- `400 Bad Request` - Invalid request
- `429 Too Many Requests` - Triggered too frequently
- `503 Service Unavailable` - Coordination disabled

---

### Get Latest Standup Results

#### `GET /api/coordination/standup/latest`

Fetch the most recent standup result.

**Request:**
```bash
curl -X GET http://localhost:7777/api/coordination/standup/latest
```

**Response (200 OK):**
```json
{
  "request_id": "standup-2026-02-14-0900",
  "responses": [
    {
      "agent_id": "k8s-monitor",
      "what_i_did": "Monitored 12 pods across 3 namespaces",
      "what_im_doing": "Diagnosing pod logs",
      "blockers": [],
      "token_count": 284,
      "timestamp": "2026-02-14T09:00:15.456Z"
    }
  ],
  "summary": "Operational status: 2 agents active",
  "triggered_at": "2026-02-14T09:00:00.000Z"
}
```

**Status Codes:**
- `200 OK` - Results available
- `204 No Content` - No standup results yet
- `503 Service Unavailable` - Coordination disabled

---

## Conversation Endpoints

### Create Conversation Session

#### `POST /api/conversation/session`

Create a new conversational agent creation session.

**Request:**
```bash
curl -X POST http://localhost:7777/api/conversation/session \
  -H "Content-Type: application/json" \
  -d '{
    "user_description": "I need a K8s expert that can diagnose cluster issues",
    "agent_type": "specialist"
  }'
```

**Response (201 Created):**
```json
{
  "session_id": "conv-sess-abc123",
  "user_message": "I need a K8s expert...",
  "assistant_message": "I'll help create this agent. Let me ask some clarifying questions...",
  "next_question": "What specific K8s operations should this agent focus on?",
  "status": "awaiting_user_input"
}
```

**Status Codes:**
- `201 Created` - Session created
- `400 Bad Request` - Invalid request

---

### Get Conversation Session

#### `GET /api/conversation/session/:id`

Get current state of a conversation session.

**Request:**
```bash
curl -X GET http://localhost:7777/api/conversation/session/conv-sess-abc123
```

**Response (200 OK):**
```json
{
  "session_id": "conv-sess-abc123",
  "messages": [
    {
      "role": "user",
      "content": "I need a K8s expert that can diagnose cluster issues",
      "timestamp": "2026-02-14T09:00:00.000Z"
    },
    {
      "role": "assistant",
      "content": "I'll help create this agent...",
      "timestamp": "2026-02-14T09:00:01.000Z"
    }
  ],
  "current_question": "What specific K8s operations should this agent focus on?",
  "status": "awaiting_user_input"
}
```

**Status Codes:**
- `200 OK` - Session found
- `404 Not Found` - Session not found

---

### Send Conversation Message

#### `POST /api/conversation/message`

Send user response in ongoing conversation.

**Request:**
```bash
curl -X POST http://localhost:7777/api/conversation/message \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "conv-sess-abc123",
    "message": "Diagnostics, debugging, and performance monitoring"
  }'
```

**Response (200 OK):**
```json
{
  "session_id": "conv-sess-abc123",
  "user_message": "Diagnostics, debugging...",
  "assistant_response": "Great, that helps...",
  "next_question": "Should this agent have access to logs?",
  "status": "awaiting_user_input"
}
```

**Status Codes:**
- `200 OK` - Message processed
- `404 Not Found` - Session not found
- `400 Bad Request` - Invalid message

---

### Confirm and Create Agent

#### `POST /api/conversation/confirm`

Confirm specifications and create the agent.

**Request:**
```bash
curl -X POST http://localhost:7777/api/conversation/confirm \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "conv-sess-abc123",
    "final_name": "k8s-diagnostics"
  }'
```

**Response (201 Created):**
```json
{
  "session_id": "conv-sess-abc123",
  "agent_id": "k8s-diagnostics",
  "agent_name": "Kubernetes Diagnostics Expert",
  "agent_config": {
    "model": "google:gemini-2.5-flash",
    "capabilities": ["diagnostics", "debugging", "monitoring"],
    "description": "Specializes in K8s cluster diagnostics..."
  },
  "config_written_to": "agents/k8s-diagnostics.yaml",
  "status": "created"
}
```

**Status Codes:**
- `201 Created` - Agent created
- `404 Not Found` - Session not found
- `409 Conflict` - Agent name already exists

---

### Cancel Conversation

#### `POST /api/conversation/cancel`

Cancel an ongoing conversation session without creating an agent.

**Request:**
```bash
curl -X POST http://localhost:7777/api/conversation/cancel \
  -H "Content-Type: application/json" \
  -d '{"session_id": "conv-sess-abc123"}'
```

**Response (200 OK):**
```json
{
  "session_id": "conv-sess-abc123",
  "status": "cancelled",
  "message": "Conversation cancelled"
}
```

**Status Codes:**
- `200 OK` - Cancelled
- `404 Not Found` - Session not found

---

## WebSocket Events

### Connection

Connect to `/ws` to receive real-time events. Server sends existing event backlog (~100 recent events) upon connection, then streams real-time events.

**Connection:**
```javascript
const ws = new WebSocket('ws://localhost:7777/ws');

ws.onopen = () => {
  console.log('Connected to event stream');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  // Handle event based on type
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  // Reconnect with exponential backoff (max 30s)
};
```

### Event Types

#### HeartbeatResponse

Agent responded to heartbeat check.

```typescript
interface HeartbeatResponseEvent {
  type: "coordination_activity";
  coordination_activity: {
    type: "HeartbeatResponse";
    agent_id: string;
    status: "Healthy" | "Degraded" | "Unresponsive";
    response_ms: number;
    timestamp: string;  // ISO 8601
  };
}
```

**UI Action:** Update agent health indicator with status and latency.

---

#### HeartbeatTimeout

Agent failed to respond to heartbeat.

```typescript
interface HeartbeatTimeoutEvent {
  type: "coordination_activity";
  coordination_activity: {
    type: "HeartbeatTimeout";
    agent_id: string;
    consecutive_misses: number;
    timestamp: string;
  };
}
```

**UI Action:** Mark agent as "Unresponsive" with pulsing indicator.

---

#### StandupResponse

Agent responded to standup prompt.

```typescript
interface StandupResponseEvent {
  type: "coordination_activity";
  coordination_activity: {
    type: "StandupResponse";
    request_id: string;
    agent_id: string;
    what_i_did: string;
    what_im_doing: string;
    blockers: string[];
    token_count: number;
    timestamp: string;
  };
}
```

**UI Action:** Add/update response card in standup feed.

---

#### StandupSummary

All agents have responded to standup (or timeout reached).

```typescript
interface StandupSummaryEvent {
  type: "coordination_activity";
  coordination_activity: {
    type: "StandupSummary";
    request_id: string;
    summary: string;
    responses_collected: number;
    total_agents: number;
    timestamp: string;
  };
}
```

**UI Action:** Display summary and mark standup complete.

---

#### ExecutionEvent

Generic event when an agent executes (webhook message received, command triggered).

```typescript
interface ExecutionEvent {
  type: "execution_event";
  execution: {
    id: string;
    agent_id: string;
    platform: string;  // "slack", "discord", etc
    user_id: string;
    command: string;
    status: "started" | "running" | "completed" | "failed";
    timestamp: string;
  };
}
```

---

#### IntroductionEvent

Agent introduction on cold start.

```typescript
interface IntroductionEvent {
  type: "introduction";
  introduction: {
    agent_id: string;
    agent_name: string;
    intro_message: string;
    capabilities: string[];
    timestamp: string;
  };
}
```

---

## Error Handling

### HTTP Error Responses

All endpoints may return standardized error responses:

```json
{
  "error": "Service Unavailable",
  "message": "Coordination protocols not configured",
  "code": "COORD_DISABLED",
  "request_id": "req-123456"
}
```

### Common Error Codes

| Code | HTTP | Message | Action |
|------|------|---------|--------|
| `COORD_DISABLED` | 503 | Coordination not configured | Show "Not enabled" state |
| `INVALID_MODE` | 400 | Invalid coordination mode | Validate input |
| `RATE_LIMIT` | 429 | Too many requests | Wait and retry |
| `AGENT_NOT_FOUND` | 404 | Agent not found | Check agent ID |
| `SESSION_EXPIRED` | 410 | Conversation session expired | Create new session |
| `INTERNAL_ERROR` | 500 | Server error | Log request_id |

### WebSocket Disconnection

1. Browser handles reconnection automatically
2. Redux state persists (last known values)
3. UI shows "Reconnecting..." indicator
4. Exponential backoff (max 30s between attempts)

---

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `GET /api/config/*` | 120 req/min | Per IP |
| `GET /api/agents/*/metrics` | 60 req/min | Per IP |
| `GET /api/coordination/health` | 60 req/min | Per IP |
| `GET /api/coordination/metrics` | 120 req/min | Per IP |
| `POST /api/coordination/standup/trigger` | 1 req / 10 sec | Per client |
| `POST /api/coordination/mode` | 5 req / min | Per IP |
| `POST /webhook/:platform` | 1000 req/min | Per platform |

**Behavior:** Requests exceeding limit return `429 Too Many Requests` with `Retry-After` header.

---

## Polling Strategy (Frontend)

```typescript
// Health & Standup: Event-driven (WebSocket only)
// No polling needed - uses real-time events

// Metrics: Polled (REST) - 30 second interval
setInterval(() => {
  fetch('/api/coordination/metrics')
    .then(r => r.json())
    .then(metrics => dispatch(updateMetrics(metrics)))
}, 30000);

// Config: Polled (REST) - 5 minute interval or on app start
setInterval(() => {
  fetch('/api/config/agents')
    .then(r => r.json())
    .then(agents => dispatch(updateAgents(agents)))
}, 300000);

// WebSocket: Connect once, listen forever
const ws = new WebSocket('ws://localhost:7777/ws');
```

---

## Authentication

**Current:** No authentication (localhost/internal network only)

**Future (Phase 8+):**
- Device pairing via mTLS
- API key tokens for external integrations
- OAuth2 for enterprise deployments

---

## Integration Examples

### React Hook for Config API

```typescript
import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';

const useConfigAPI = () => {
  const dispatch = useDispatch();
  const { agents, tools, version } = useSelector(state => state.config);

  useEffect(() => {
    // Load config on mount
    Promise.all([
      fetch('/api/config/agents').then(r => r.json()),
      fetch('/api/config/tools').then(r => r.json()),
      fetch('/api/config/version').then(r => r.json()),
    ]).then(([agents, tools, version]) => {
      dispatch({
        type: 'CONFIG_LOADED',
        payload: { agents, tools, version }
      });
    });
  }, [dispatch]);

  return { agents, tools, version };
};
```

### React Hook for WebSocket

```typescript
useEffect(() => {
  const ws = new WebSocket('ws://localhost:7777/ws');

  ws.onmessage = (event) => {
    const message = JSON.parse(event.data);

    if (message.type === 'coordination_activity') {
      switch (message.coordination_activity.type) {
        case 'HeartbeatResponse':
          dispatch(updateAgentHealth(message.coordination_activity));
          break;
        case 'StandupResponse':
          dispatch(addStandupResponse(message.coordination_activity));
          break;
        // ... handle other events
      }
    }
  };

  return () => ws.close();
}, [dispatch]);
```

---

## Status Codes Reference

| Code | Meaning | Action |
|------|---------|--------|
| 200 | OK | Use response data |
| 201 | Created | Resource created |
| 202 | Accepted | Processing queued |
| 204 | No Content | Success, no data |
| 400 | Bad Request | Fix request body |
| 401 | Unauthorized | Add auth |
| 404 | Not Found | Verify ID/path |
| 409 | Conflict | Resource exists |
| 429 | Too Many Requests | Wait and retry |
| 500 | Internal Server Error | Contact admin |
| 503 | Service Unavailable | Feature disabled |

---

## Changelog

**v1.0 (2026-02-14)**
- Complete API specification for all endpoints
- 7 API families documented
- 20+ endpoints with schemas
- 5+ WebSocket event types
- Rate limits and polling strategy
- Error handling and status codes

---

**For builder.io:** This specification is your complete contract. All endpoints are stable and production-ready. Use the schemas, examples, and WebSocket event types to build the frontend.
