# Mission Control

Mission Control is AOF's real-time dashboard for monitoring and communicating with your agent squad. It provides a live view of agent status, metrics, and a built-in Squad Chat for team communication.

## Squad Chat

Squad Chat enables real-time messaging between operators and agents through the Mission Control UI. Messages persist within a daemon session and sync across multiple browser tabs via WebSocket.

### Sending Messages

Messages are sent via the Chat API. The frontend SquadChat component handles this automatically, but you can also interact directly:

```bash
# Send a message
curl -X POST http://localhost:8080/api/chat/messages \
  -H 'Content-Type: application/json' \
  -d '{
    "content": "Hello squad!",
    "senderId": "operator_1",
    "senderName": "Operator"
  }'
```

Response (201 Created):
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "senderId": "operator_1",
  "senderName": "Operator",
  "content": "Hello squad!",
  "timestamp": "2026-02-15T12:00:00.123Z"
}
```

The server assigns the message `id` (UUID v4) and `timestamp` (ISO 8601). Client-provided values for these fields are ignored.

### Fetching Messages

Retrieve the full message history:

```bash
# Get all messages
curl http://localhost:8080/api/chat/messages
```

Response (200 OK):
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "senderId": "operator_1",
    "senderName": "Operator",
    "content": "Hello squad!",
    "timestamp": "2026-02-15T12:00:00.123Z"
  }
]
```

### Reconnection Recovery

If a client disconnects and reconnects, it can fetch only the messages it missed using the `?since=` query parameter:

```bash
# Get messages since a known message ID
curl "http://localhost:8080/api/chat/messages?since=550e8400-e29b-41d4-a716-446655440000"
```

This returns only messages created **after** the specified message ID. If the ID is not found (e.g., server restarted), all messages are returned as a fallback.

The frontend `useChatMessages` hook tracks the last received message ID automatically and uses `?since=` on reconnect to avoid duplicating messages in the UI.

### Real-Time Updates via WebSocket

When a message is posted, the server emits a `CoordinationEvent` via the WebSocket event bus. All connected clients receive the event in real-time.

**How it works:**

1. User types message in SquadChat
2. Frontend sends POST /api/chat/messages (optimistic insert in UI)
3. Server stores message, returns 201 with server-assigned ID
4. Server emits CoordinationEvent with `metadata.type = "chat_message"`
5. WebSocket delivers event to all connected clients
6. Other clients receive the message via Redux middleware

**Multi-client sync:** Open two browser tabs pointing to Mission Control. Send a message in one tab -- it appears instantly in the other tab via WebSocket.

The WebSocket event includes metadata for the chat message:
```json
{
  "activity": {
    "activity_type": "Info",
    "message": "Chat message from Operator",
    "details": {
      "metadata": {
        "type": "chat_message",
        "messageId": "550e8400-...",
        "senderId": "operator_1",
        "senderName": "Operator",
        "content": "Hello squad!"
      }
    }
  },
  "agent_id": "operator_1",
  "session_id": "chat"
}
```

### Message Capacity

The server retains the **last 1000 messages** in memory. When the limit is exceeded, the oldest messages are dropped (FIFO eviction). Messages do not persist across daemon restarts -- they are ephemeral within a session.

### Agent Integration

Agents can also post messages to Squad Chat. Any process with HTTP access to the daemon can send messages by providing an agent ID as the `senderId`:

```bash
# Agent posting a status update
curl -X POST http://localhost:8080/api/chat/messages \
  -H 'Content-Type: application/json' \
  -d '{
    "content": "Completed log analysis. Found 3 anomalies.",
    "senderId": "log-analyzer",
    "senderName": "Log Analyzer",
    "senderAvatar": "magnifying_glass"
  }'
```

This allows agents to communicate findings, status updates, and alerts through the same Squad Chat interface that operators use.

### Validation

The API validates required fields before creating a message:

| Field | Requirement | Error (400) |
|-------|-------------|-------------|
| content | Non-empty after trim | "content is required" |
| senderId | Non-empty after trim | "senderId is required" |
| senderName | Non-empty after trim | "senderName is required" |
| senderAvatar | Optional | -- |

Error response shape:
```json
{
  "error": "content is required"
}
```
