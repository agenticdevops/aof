# Mission Control Tasks API - Internal Design

## Overview

The Tasks API provides backend endpoints for the Mission Control Kanban board. The frontend (`web-ui/src/hooks/useTaskManagement.ts`) performs optimistic updates and relies on these endpoints for persistence and multi-client synchronization.

## Endpoints

### GET /api/tasks

Returns all tasks as a JSON array.

**Response:** `200 OK`
```json
[
  {
    "id": "task-001",
    "title": "Monitor k8s cluster health",
    "description": "Periodic health check of production cluster",
    "lane": "in-progress",
    "assignedTo": "sentinel",
    "version": 1,
    "createdAt": "2025-01-15T10:00:00Z",
    "updatedAt": "2025-01-15T10:00:00Z",
    "status": "active",
    "priority": "high",
    "tags": ["monitoring", "k8s"],
    "dueDate": null
  }
]
```

### POST /api/tasks

Creates a new task with auto-generated ID and version=1.

**Request:**
```json
{
  "title": "New task title",
  "description": "Optional description",
  "lane": "backlog",
  "assignedTo": "agent-name",
  "priority": "medium",
  "tags": ["tag1", "tag2"]
}
```

- `title` is required (non-empty).
- `description` defaults to empty string if omitted.
- `lane` defaults to `"backlog"` if omitted. Must be one of: `backlog`, `assigned`, `in-progress`, `review`, `done`.
- All other fields are optional.

**Response:** `201 Created`
```json
{
  "id": "a1b2c3d4-...",
  "title": "New task title",
  "description": "",
  "lane": "backlog",
  "assignedTo": null,
  "version": 1,
  "createdAt": "2025-01-15T12:00:00Z",
  "updatedAt": "2025-01-15T12:00:00Z",
  "status": "pending",
  "priority": null,
  "tags": null,
  "dueDate": null
}
```

**Error Responses:**
- `400 Bad Request` - Empty title or invalid lane value.

### POST /api/tasks/move

Moves a task to a different lane with version-based optimistic concurrency control.

**Request:**
```json
{
  "taskId": "task-001",
  "newLane": "review",
  "version": 1
}
```

**Response (success):** `200 OK`
```json
{
  "task": { "...updated task with version incremented..." },
  "success": true,
  "error": null
}
```

**Response (version conflict):** `409 Conflict`
```json
{
  "task": { "...current server-side task state..." },
  "success": false,
  "error": "Version conflict: expected 1, found 2"
}
```

**Response (not found):** `404 Not Found`
```json
{
  "error": "Task not found: task-xyz"
}
```

**Response (bad request):** `400 Bad Request`
```json
{
  "error": "Invalid lane: 'unknown'. Must be one of: backlog, assigned, in-progress, review, done"
}
```

## In-Memory Storage Design

```
TaskStore {
    tasks: Vec<Task>
}

TasksState {
    store: Arc<RwLock<TaskStore>>   // Thread-safe shared state
    event_bus: Option<Arc<EventBroadcaster>>  // For WebSocket emission
}
```

- `Arc<RwLock<TaskStore>>` provides concurrent read access and exclusive write access.
- Multiple GET requests can read simultaneously.
- POST (create/move) acquires write lock, performs mutation, releases lock.
- Tasks persist in memory for the lifetime of the daemon process.

### Seed Data

On startup, `TaskStore::new()` seeds 5 sample tasks across different lanes so the KanbanBoard shows content on first load:

1. "Monitor k8s cluster health" - `in-progress` lane, priority: high
2. "Review deployment pipeline" - `backlog` lane, priority: medium
3. "Update security patches" - `assigned` lane, priority: critical
4. "Analyze error logs" - `review` lane, priority: medium
5. "Optimize database queries" - `done` lane, priority: low

## Version-Based Optimistic Concurrency Control

1. Client reads task (version=N).
2. Client sends `POST /api/tasks/move` with `version=N`.
3. Server checks: if `task.version == request.version`, move succeeds.
   - Server increments version to N+1, updates timestamp.
   - Returns updated task with new version.
4. If `task.version != request.version` (another client moved it first):
   - Server returns 409 with current task state.
   - Client can inspect server version and decide whether to retry.

## Status Transitions on Lane Change

- Moving to `"done"` sets `status = "completed"`.
- Moving from `"backlog"` to any other lane sets `status = "active"`.
- All other moves preserve the current status (unless moving to done).

## WebSocket Event Emission

On successful task mutation (create or move), the handler emits a `CoordinationEvent` via the `EventBroadcaster`:

- **TASK_CREATED**: Emitted when `POST /api/tasks` succeeds.
- **TASK_MOVED**: Emitted when `POST /api/tasks/move` succeeds.

These events flow through the existing WebSocket infrastructure to all connected clients, enabling multi-client real-time sync.

Event payload (embedded in CoordinationEvent metadata):
```json
{
  "event_type": "TASK_MOVED",
  "task_id": "task-001",
  "from_lane": "backlog",
  "to_lane": "in-progress",
  "version": 2
}
```

## Error Response Shape

All error responses use a consistent JSON shape:
```json
{
  "error": "Human-readable error description"
}
```

For 409 Conflict, the response includes the current task for client-side resolution:
```json
{
  "task": { "...server-side task..." },
  "success": false,
  "error": "Version conflict: expected 1, found 3"
}
```

## File Location

- Implementation: `crates/aofctl/src/api/tasks.rs`
- Module registration: `crates/aofctl/src/api/mod.rs`
- Route wiring: `crates/aofctl/src/commands/serve.rs`

## Frontend Contract (DO NOT MODIFY)

The backend must match these TypeScript types exactly:
- `web-ui/src/types/tasks.ts` - Type definitions
- `web-ui/src/hooks/useTaskManagement.ts` - API calls
- `web-ui/src/store/tasksSlice.ts` - Redux optimistic updates

All JSON field names use camelCase (e.g., `assignedTo`, `createdAt`, `taskId`, `newLane`).
