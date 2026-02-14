# AOF Mission Control API Specification

**Version:** 1.0
**Status:** Active (Phase 7)
**Last Updated:** 2026-02-14
**Audience:** Frontend Developers, API Integrators, builder.io

---

## Overview

This specification defines all REST API endpoints and WebSocket message formats for Mission Control UI. Use this document for:
- **Frontend developers** building UI components
- **API integrators** consuming endpoints from external systems
- **builder.io** configuration and component specs

---

## Base URLs

| Environment | URL | Port | Protocol |
|-------------|-----|------|----------|
| Local Development | `http://localhost` | 7777 | HTTP/WS |
| Docker | `http://aof-daemon` | 7777 | HTTP/WS |
| Production | `https://aof-api.example.com` | 443 | HTTPS/WSS |

**WebSocket endpoint:** `/ws` (upgrades from HTTP)

---

## API Endpoints

### Coordination Health Check

#### `GET /api/coordination/health`

Fetch current agent health status.

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
    "frequency_secs": 30,
    "timeout_secs": 60
  },
  "coordination_enabled": true
}
```

**Status Codes:**
- `200 OK` - Health data returned
- `503 Service Unavailable` - Coordination disabled or not configured

**Response Schema:**
```typescript
interface AgentHealthRecord {
  agent_id: string;
  status: "Healthy" | "Degraded" | "Unresponsive";
  last_heartbeat: string | null;  // ISO 8601 timestamp
  consecutive_misses: number;     // Count of consecutive missed heartbeats
  last_response_ms: number | null; // Latency in milliseconds
  degraded_reason?: string;       // Human-readable reason if Degraded
}

interface HeartbeatHealthResponse {
  agents: AgentHealthRecord[];
  heartbeat_config: {
    frequency_secs: number;
    timeout_secs: number;
  };
  coordination_enabled: boolean;
}
```

---

### Standup Results

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
      "what_i_did": "Monitored 12 pods across 3 namespaces, detected 1 CrashLoopBackOff",
      "what_im_doing": "Diagnosing pod logs for failed deployment",
      "blockers": [],
      "token_count": 284,
      "timestamp": "2026-02-14T09:00:15.456Z"
    },
    {
      "agent_id": "log-analyzer",
      "what_i_did": "Parsed 45K log lines from failed deployment",
      "what_im_doing": "Identifying error patterns and root cause",
      "blockers": ["Waiting for k8s-monitor's diagnosis"],
      "token_count": 512,
      "timestamp": "2026-02-14T09:00:30.789Z"
    }
  ],
  "summary": "Operational status: 2 agents active. 1 blocked on K8s diagnosis. No critical issues.",
  "triggered_at": "2026-02-14T09:00:00.000Z"
}
```

**Status Codes:**
- `200 OK` - Standup results available
- `204 No Content` - No standup results yet
- `503 Service Unavailable` - Coordination disabled

**Response Schema:**
```typescript
interface StandupResponseRecord {
  agent_id: string;
  what_i_did: string;           // Previous work summary
  what_im_doing: string;        // Current task
  blockers: string[];           // List of blockers
  token_count: number;          // Tokens consumed by this agent's response
  timestamp: string;            // ISO 8601 when response was recorded
}

interface StandupResult {
  request_id: string;           // Unique standup ID
  responses: StandupResponseRecord[];
  summary?: string;             // Optional Sonnet-generated summary
  triggered_at: string;         // ISO 8601 when standup was triggered
}
```

---

### Coordination Metrics

#### `GET /api/coordination/metrics`

Fetch coordination token overhead and mode status. **Polled every 30 seconds** by frontend.

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

**Response Schema:**
```typescript
interface CoordinationMetrics {
  coordination_tokens: number;    // Total tokens spent on heartbeat + standup
  production_tokens: number;      // Tokens spent on production work
  overhead_percent: number;       // (coordination / (coordination + production)) * 100
  heartbeat_tokens: number;       // Tokens spent on heartbeats in window
  standup_tokens: number;         // Tokens spent on standups in window
  current_mode: "Full" | "Standard" | "Reduced" | "HeartbeatOnly" | "Disabled";
  auto_degrade_enabled: boolean;  // Whether auto-degradation is active
  max_overhead_percent: number;   // Threshold for auto-degradation (default 30)
  window_start: string;           // ISO 8601 start of metrics window
}
```

---

### Manual Standup Trigger

#### `POST /api/coordination/standup/trigger`

Manually trigger an immediate standup (instead of waiting for scheduled time).

**Request:**
```bash
curl -X POST http://localhost:7777/api/coordination/standup/trigger \
  -H "Content-Type: application/json" \
  -d '{"reason": "Incident response check-in"}'
```

**Request Body:**
```typescript
interface TriggerStandupRequest {
  reason?: string;  // Optional reason for standup (logged, not sent to agents)
}
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
- `202 Accepted` - Standup triggered, will process asynchronously
- `400 Bad Request` - Invalid request body
- `429 Too Many Requests` - Standups triggered too frequently (<10 seconds apart)
- `503 Service Unavailable` - Coordination disabled

**Response Schema:**
```typescript
interface TriggerStandupResponse {
  request_id: string;       // ID for tracking this standup
  status: "processing";     // Always "processing" initially
  expected_completion_ms: number; // Estimated time to collect all responses
}
```

---

### Coordination Mode Override

#### `POST /api/coordination/mode`

Force coordination mode change (for testing or manual control).

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
  reason?: string;  // Optional reason (logged)
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
- `200 OK` - Mode changed successfully
- `400 Bad Request` - Invalid mode value
- `503 Service Unavailable` - Coordination disabled

**Response Schema:**
```typescript
interface SetModeResponse {
  previous_mode: string;
  new_mode: string;
  auto_degrade_enabled: boolean;  // Auto-degradation disabled after manual override
  reason?: string;
}
```

---

## WebSocket Events

Connect to `/ws` and listen for coordination events. The WebSocket connection is **bidirectional** but currently events flow **server → client only**.

### Connection Flow

1. **Client connects** to `ws://localhost:7777/ws`
2. **Server sends** existing event backlog (~100 recent events)
3. **Server streams** real-time events as they occur

### Event Types

#### HeartbeatResponse

Received when an agent responds to a heartbeat check.

```typescript
interface CoordinationWebSocketMessage {
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

**UI Action:** Update agent health card with new status and latency.

---

#### HeartbeatTimeout

Received when an agent fails to respond to a heartbeat.

```typescript
interface CoordinationWebSocketMessage {
  type: "coordination_activity";
  coordination_activity: {
    type: "HeartbeatTimeout";
    agent_id: string;
    consecutive_misses: number;
    timestamp: string;  // ISO 8601
  };
}
```

**UI Action:** Mark agent as "Unresponsive" with pulsing red indicator.

---

#### StandupResponse

Received as each agent responds to standup prompt.

```typescript
interface CoordinationWebSocketMessage {
  type: "coordination_activity";
  coordination_activity: {
    type: "StandupResponse";
    request_id: string;
    agent_id: string;
    what_i_did: string;
    what_im_doing: string;
    blockers: string[];
    token_count: number;
    timestamp: string;  // ISO 8601
  };
}
```

**UI Action:** Add or update standup response card in feed. Show progressive responses as they arrive.

---

#### StandupSummary

Received after all agents have responded to standup (or timeout reached).

```typescript
interface CoordinationWebSocketMessage {
  type: "coordination_activity";
  coordination_activity: {
    type: "StandupSummary";
    request_id: string;
    summary: string;
    responses_collected: number;
    total_agents: number;
    timestamp: string;  // ISO 8601
  };
}
```

**UI Action:** Display summary at top of standup feed. Mark standup as "complete".

---

## Error Handling

### HTTP Error Responses

All endpoints may return:

```json
{
  "error": "Service Unavailable",
  "message": "Coordination protocols not configured",
  "code": "COORD_DISABLED",
  "request_id": "req-123456"
}
```

**Common Error Codes:**

| Code | HTTP | Message | Action |
|------|------|---------|--------|
| `COORD_DISABLED` | 503 | Coordination not configured | Show "Coordination not enabled" UI state |
| `INVALID_MODE` | 400 | Invalid coordination mode | Validate mode selection before POST |
| `RATE_LIMIT` | 429 | Standup triggered too recently | Disable trigger button for 10 seconds |
| `INTERNAL_ERROR` | 500 | Server error | Show generic error, log request_id |

### WebSocket Disconnection

If WebSocket disconnects:
1. **Browser handles reconnection** automatically (exponential backoff, max 30s)
2. **Redux state persists** (last known values)
3. **UI shows "Reconnecting..." indicator**

---

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `GET /api/coordination/health` | 60 req/min | Per IP |
| `GET /api/coordination/standup/latest` | 60 req/min | Per IP |
| `GET /api/coordination/metrics` | 120 req/min | Per IP (30s polling) |
| `POST /api/coordination/standup/trigger` | 1 req / 10 sec | Per client |
| `POST /api/coordination/mode` | 5 req / min | Per IP |

**Behavior:** Requests exceeding limit return `429 Too Many Requests` with `Retry-After` header.

---

## Polling Strategy

### Recommended Frontend Polling

```typescript
// Health: Event-driven (WebSocket)
// No polling needed - uses heartbeat events

// Metrics: Polled (REST)
setInterval(() => {
  fetch('/api/coordination/metrics')
    .then(r => r.json())
    .then(metrics => dispatch(setMetrics(metrics)))
}, 30000);  // 30 second interval

// Standup: Event-driven (WebSocket)
// No polling needed - uses standup events
```

**Rationale:**
- **Health & Standup** are bursty (events come at specific times)
- **Metrics** change gradually (token counts) - polling is acceptable
- This minimizes server load while keeping UI responsive

---

## Authentication

**Current:** No authentication (localhost/internal network only)

**Future (Phase 8+):**
- Device pairing via mTLS (see `docs/concepts/device-security.md`)
- API key tokens for external integrations
- OAuth2 for enterprise deployments

---

## Examples

### React Hook Usage

```typescript
// useCoordination hook consumes these endpoints
const useCoordination = () => {
  // Initial fetch
  useEffect(() => {
    fetch('/api/coordination/health').then(...)
    fetch('/api/coordination/standup/latest').then(...)
    fetch('/api/coordination/metrics').then(...)
  }, []);

  // Polling for metrics
  useEffect(() => {
    const interval = setInterval(() => {
      fetch('/api/coordination/metrics').then(...)
    }, 30000);
    return () => clearInterval(interval);
  }, []);

  // WebSocket for real-time events
  useWebSocket(() => {
    onMessage((event) => {
      if (event.type === 'coordination_activity') {
        // Handle HeartbeatResponse, StandupResponse, etc.
      }
    });
  });

  return { health, latestStandup, metrics, ... };
};
```

---

## Status Codes Reference

| Code | Meaning | Action |
|------|---------|--------|
| 200 | OK | Use response data |
| 202 | Accepted | Operation queued, check later |
| 204 | No Content | Success, but no data to return |
| 400 | Bad Request | Fix request body/params |
| 429 | Too Many Requests | Wait and retry (see Retry-After header) |
| 500 | Internal Server Error | Log error, show generic message |
| 503 | Service Unavailable | Coordination disabled, show setup UI |

---

## Changelog

**v1.0 (2026-02-14)**
- Initial API specification
- 5 endpoints documented
- 4 WebSocket event types
- Rate limits and polling strategy defined

---

**For builder.io:** Use this spec as your contract. All endpoints and event formats are stable and ready for implementation.
