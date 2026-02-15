# Mission Control Chat API - Internal Design Document

## Overview

The Chat API provides HTTP endpoints for the Mission Control SquadChat component. Messages are stored in-memory on the server (per daemon session) and broadcast via WebSocket for real-time multi-client sync.

## Endpoints

### GET /api/chat/messages

Returns the full message history as a JSON array of `ChatMessage` objects.

**Query Parameters:**
- `since` (optional, string): Message ID to resume from. Returns only messages after the specified ID. Used for reconnection recovery. If the ID is not found, all messages are returned (graceful degradation).

**Response:** `200 OK`
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "senderId": "user_1",
    "senderName": "Operator",
    "senderAvatar": null,
    "content": "Hello squad!",
    "timestamp": "2026-02-15T12:00:00.000Z",
    "threadId": null,
    "version": null
  }
]
```

### POST /api/chat/messages

Creates a new chat message. Server assigns UUID and timestamp.

**Request Body:**
```json
{
  "content": "Hello squad!",
  "senderId": "user_1",
  "senderName": "Operator",
  "senderAvatar": null
}
```

**Response:** `201 Created`
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "senderId": "user_1",
  "senderName": "Operator",
  "senderAvatar": null,
  "content": "Hello squad!",
  "timestamp": "2026-02-15T12:00:00.123Z",
  "threadId": null,
  "version": null
}
```

**Error Response:** `400 Bad Request`
```json
{
  "error": "content is required"
}
```

## Data Types

### ChatMessage (matches TypeScript interface exactly)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID v4) | yes | Server-assigned unique ID |
| senderId | string | yes | Agent or user ID |
| senderName | string | yes | Display name |
| senderAvatar | string? | no | Avatar URL or emoji |
| content | string | yes | Message body (supports markdown) |
| timestamp | string (ISO 8601) | yes | Server-assigned timestamp |
| threadId | string? | no | Optional thread for reply threading |
| version | number? | no | Optional version for conflict resolution |

All field names use **camelCase** in JSON (Rust `#[serde(rename_all = "camelCase")]`).

### SendMessageRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| content | string | yes | Message body (must be non-empty after trim) |
| senderId | string | yes | Sender identifier (must be non-empty after trim) |
| senderName | string | yes | Display name (must be non-empty after trim) |
| senderAvatar | string? | no | Avatar URL or emoji |

## Storage Design

### ChatStore (In-Memory)

```
Arc<RwLock<ChatStore>>
  - messages: Vec<ChatMessage>
  - max_capacity: 1000
```

- **Thread Safety:** `Arc<RwLock<ChatStore>>` for concurrent read/write access.
- **Capacity:** Maximum 1000 messages retained in memory. When limit is exceeded, oldest messages are dropped (FIFO eviction).
- **Ordering:** Messages are stored in chronological order (append-only). `get_all()` returns messages in insertion order.
- **Deduplication:** Server-assigned UUIDs prevent duplicate IDs. Frontend handles optimistic deduplication via Redux reducer.
- **Persistence:** In-memory only. Messages are lost when daemon restarts. This is acceptable for v1 (chat is ephemeral coordination).

### ChatState

Shared state injected into Axum handlers:
- `store: Arc<RwLock<ChatStore>>` - The message store
- `event_bus: Option<Arc<EventBroadcaster>>` - For WebSocket broadcast

## WebSocket Event Emission

When a new message is created via POST, a `CoordinationEvent` is emitted to the `EventBroadcaster`:
- `activity`: `ActivityEvent::info("Chat message from {senderName}")` with metadata containing message details
- `agent_id`: `message.sender_id`
- `session_id`: Static `"chat"` session
- `coordination_activity`: None (uses activity metadata for chat data)

WebSocket clients receive the raw `CoordinationEvent` JSON. The frontend `useWebSocket` hook processes events and the `chatSlice` reducer deduplicates by message ID.

## Error Handling

Errors follow the standard error shape: `{ "error": "message" }`.

| Condition | Status | Error Message |
|-----------|--------|---------------|
| Empty content (after trim) | 400 | "content is required" |
| Empty senderId (after trim) | 400 | "senderId is required" |
| Empty senderName (after trim) | 400 | "senderName is required" |

## Message Retention

- Maximum 1000 messages in memory
- When capacity is exceeded, oldest messages are dropped
- No disk persistence (ephemeral within daemon session)
- Future enhancement: optional SQLite persistence
