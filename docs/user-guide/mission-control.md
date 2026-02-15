# Mission Control

Mission Control is AOF's real-time dashboard for monitoring and communicating with your agent squad. It provides a live view of agent status, metrics, a Kanban board for task management, and a built-in Squad Chat for team communication.

## Task Management (Kanban Board)

The Kanban board lets you organize and track tasks across five lanes:

| Lane | Description |
|------|-------------|
| **Backlog** | Tasks waiting to be picked up |
| **Assigned** | Tasks assigned to an agent but not yet started |
| **In Progress** | Tasks actively being worked on |
| **Review** | Tasks under review |
| **Done** | Completed tasks |

### How It Works

Tasks flow through lanes from left to right. You can drag and drop tasks between lanes in the UI. Each move is synchronized with the backend using optimistic concurrency control -- the UI updates immediately while the server processes the change in the background.

### API Endpoints

#### List All Tasks

```bash
curl http://localhost:8080/api/tasks
```

Returns a JSON array of all tasks. On first startup, 5 sample tasks are seeded across different lanes so the board shows content immediately.

#### Create a Task

```bash
curl -X POST http://localhost:8080/api/tasks \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "Deploy staging environment",
    "description": "Set up staging cluster with latest images",
    "lane": "backlog",
    "assignedTo": "deployer",
    "priority": "high",
    "tags": ["deployment", "staging"]
  }'
```

- `title` is required (non-empty string)
- `lane` defaults to `"backlog"` if omitted; must be one of: backlog, assigned, in-progress, review, done
- All other fields are optional
- Returns the created task with a generated UUID and `version: 1`
- HTTP status: `201 Created`

#### Move a Task

```bash
curl -X POST http://localhost:8080/api/tasks/move \
  -H 'Content-Type: application/json' \
  -d '{
    "taskId": "task-001",
    "newLane": "review",
    "version": 1
  }'
```

Response (200 OK):

```json
{
  "task": {
    "id": "task-001",
    "title": "Monitor k8s cluster health",
    "lane": "review",
    "version": 2,
    "status": "active",
    "updatedAt": "2025-01-15T12:30:00Z"
  },
  "success": true
}
```

### Optimistic Concurrency Control

Every task has a `version` field that increments each time the task is modified. When you move a task, you must include the current version in your request.

When a version conflict occurs (another client moved the task first), the server returns HTTP 409 with the current server-side task state:

```json
{
  "task": { "...current server state..." },
  "success": false,
  "error": "Version conflict: task was modified by another client"
}
```

The UI automatically rolls back the optimistic update and shows an error message.

### Drag-and-Drop Flow

When you drag a task to a different lane in the Kanban board:

1. The UI immediately moves the card (optimistic update)
2. A `POST /api/tasks/move` request is sent to the server
3. On success: the optimistic state is committed with the new version
4. On 409 conflict: the card snaps back to its original position
5. On server error (5xx): the request is retried up to 3 times with exponential backoff

### Multi-Client Sync (WebSocket)

When a task is moved, the server broadcasts a `TASK_MOVED` event via WebSocket to all connected clients. Open two browser tabs to Mission Control -- drag a task in one tab, and it automatically moves in the other.

### Status Transitions

| Move | Status Change |
|------|---------------|
| Any lane to **Done** | Status becomes `completed` |
| **Backlog** to any other lane | Status becomes `active` |
| Other moves | Status unchanged |

### Task Lifecycle Example

```
1. Create task      -> status: pending,   lane: backlog,     version: 1
2. Move to assigned -> status: active,    lane: assigned,    version: 2
3. Move to progress -> status: active,    lane: in-progress, version: 3
4. Move to review   -> status: active,    lane: review,      version: 4
5. Move to done     -> status: completed, lane: done,        version: 5
```

### Error Handling

| HTTP Status | Meaning | When |
|-------------|---------|------|
| `200 OK` | Task moved successfully | Move request with valid version |
| `201 Created` | Task created | Create request with valid title |
| `400 Bad Request` | Invalid input | Empty title, invalid lane name |
| `404 Not Found` | Task does not exist | Move request for unknown taskId |
| `409 Conflict` | Version mismatch | Another client modified the task |

### Persistence

Tasks are stored in-memory on the server. They persist across page refreshes within the same daemon session but reset when the daemon restarts. Five sample tasks are seeded on each startup for demonstration.

---

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
