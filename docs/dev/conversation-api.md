# Conversation API Documentation

Developer guide for the conversational agent creation API.

## API Endpoints

All conversation endpoints are mounted under `/api/conversation/`.

### POST /api/conversation/session

Create a new conversation session.

**Request:**
```json
{}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Status Codes:**
- `200 OK` - Session created successfully
- `500 Internal Server Error` - Session creation failed

---

### GET /api/conversation/session/:id

Get details about an existing conversation session.

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "messages": [
    {
      "role": "user",
      "content": "I need a K8s monitoring agent",
      "timestamp": "2026-02-14T07:00:00Z"
    },
    {
      "role": "assistant",
      "content": "I'll help you create a Kubernetes monitoring agent...",
      "timestamp": "2026-02-14T07:00:02Z"
    }
  ],
  "created_at": "2026-02-14T07:00:00Z"
}
```

**Status Codes:**
- `200 OK` - Session found
- `404 Not Found` - Session does not exist

---

### POST /api/conversation/message

Send a message in the conversation and get orchestrator response.

**Request:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "I need a K8s monitoring agent that checks pod health"
}
```

**Response (Specialist Result):**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "response": {
    "type": "specialist_result",
    "intent": "create_agent",
    "files": {
      "workspace/AGENTS.md": "- id: k8s-monitor\n  name: Kubernetes Monitor...",
      "workspace/SOUL.md": "# k8s-monitor\nYou are a Kubernetes monitoring expert..."
    },
    "message": "I've created a Kubernetes monitoring agent. Review the files before confirming."
  },
  "messages": [
    /* Full message history */
  ]
}
```

**Response (Clarifying Questions):**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "response": {
    "type": "clarifying_questions",
    "questions": [
      "Which Kubernetes cluster should this agent monitor?",
      "What metrics are most important to you?"
    ],
    "partial_intent": "create_agent"
  },
  "messages": [/* Full history */]
}
```

**Status Codes:**
- `200 OK` - Message processed
- `404 Not Found` - Session not found
- `500 Internal Server Error` - Orchestrator error

---

### POST /api/conversation/confirm

Confirm and persist generated files to workspace.

**Request:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{
  "files_written": [
    "workspace/AGENTS.md",
    "workspace/SOUL.md"
  ],
  "message": "Successfully created 2 files and modified 0 files"
}
```

**Status Codes:**
- `200 OK` - Files persisted successfully
- `400 Bad Request` - No pending files to confirm
- `404 Not Found` - Session not found
- `500 Internal Server Error` - Persistence failed

---

### POST /api/conversation/cancel

Cancel pending file generation and clear session state.

**Request:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{}
```

**Status Codes:**
- `200 OK` - Pending files cancelled
- `404 Not Found` - Session not found

---

## Authentication

Current version: None (local daemon only).

Future: API key authentication for remote deployments.

---

## Error Handling

All errors follow this format:

```json
{
  "error": "Session not found"
}
```

**Error Status Codes:**
- `400 Bad Request` - Invalid input (missing fields, malformed JSON)
- `404 Not Found` - Resource not found (session ID)
- `500 Internal Server Error` - Server-side error (orchestrator failure, persistence failure)

---

## WebSocket Integration

Conversation events are emitted through the existing WebSocket connection at `/ws`.

**Event Types:**
- `agent_created` - New agent persisted to workspace
- `config_changed` - Workspace files modified

**Example Event:**
```json
{
  "event_id": "...",
  "agent_id": "system",
  "timestamp": "2026-02-14T07:00:00Z",
  "activity": {
    "type": "config_changed",
    "details": {
      "files": ["workspace/AGENTS.md", "workspace/SOUL.md"]
    }
  }
}
```

The conversation API itself uses HTTP request-response (not WebSocket streaming). WebSocket is used for async notifications after file persistence.

---

## State Management (Frontend)

The web UI uses Redux for conversation state:

**Redux Slice:** `conversationSlice.ts`

**State Shape:**
```typescript
{
  sessionId: string | null,
  messages: ConversationMessage[],
  isLoading: boolean,
  pendingFiles: Record<string, string> | null,
  error: string | null,
  lastResponse: OrchestratorResponse | null
}
```

**Async Thunks:**
- `createSession()` - POST /api/conversation/session
- `sendMessage({ session_id, message })` - POST /api/conversation/message
- `confirmFiles({ session_id })` - POST /api/conversation/confirm
- `cancelPending({ session_id })` - POST /api/conversation/cancel

---

## Manual End-to-End Test

Use these steps to manually test the full conversation flow:

### 1. Start the daemon

```bash
cd /Users/gshah/work/opsflow-sh/aof
cargo run --bin aofctl -- serve --workspace ./workspace --port 8080
```

### 2. Create a session

```bash
curl -X POST http://localhost:8080/api/conversation/session \
  -H "Content-Type: application/json" \
  -d '{}'
```

**Expected Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

Save the `session_id` for subsequent requests.

### 3. Send a message

```bash
SESSION_ID="<session-id-from-step-2>"

curl -X POST http://localhost:8080/api/conversation/message \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\",
    \"message\": \"I need a K8s monitoring agent that checks pod health every 5 minutes\"
  }"
```

**Expected Response:**
- `response.type` should be `"specialist_result"`
- `response.files` should contain `workspace/AGENTS.md` and `workspace/SOUL.md`
- `response.message` should describe the generated agent

### 4. Verify files not yet written

```bash
ls -la ./workspace/
```

**Expected:** AGENTS.md and SOUL.md either don't exist or don't contain the new agent yet (pending confirmation).

### 5. Confirm files

```bash
curl -X POST http://localhost:8080/api/conversation/confirm \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\"
  }"
```

**Expected Response:**
```json
{
  "files_written": ["workspace/AGENTS.md", "workspace/SOUL.md"],
  "message": "Successfully created 2 files and modified 0 files"
}
```

### 6. Verify files written

```bash
cat ./workspace/AGENTS.md
cat ./workspace/SOUL.md
```

**Expected:** Files contain the generated agent configuration and personality.

### 7. Test UI in browser

Open http://localhost:8080/#/create-agent

**Verify:**
1. Chat interface loads
2. Welcome message shows examples
3. Type "I need a deployment agent" and click Send
4. Response appears in chat
5. If files generated, FilePreview component shows
6. Click "Confirm & Save"
7. Success message appears
8. Navigate to Dashboard (click "← Back to Dashboard")
9. New agent appears in AgentGrid

### 8. Test cancellation flow

Repeat steps 2-3, but instead of confirming:

```bash
curl -X POST http://localhost:8080/api/conversation/cancel \
  -H "Content-Type: application/json" \
  -d "{
    \"session_id\": \"$SESSION_ID\"
  }"
```

**Expected:** Pending files cleared, no files written to workspace.

---

## Extending the UI

To add new conversation features:

1. **Add new response type to OrchestratorResponse** (types/conversation.ts)
2. **Handle new type in conversationSlice** (extraReducers for sendMessage)
3. **Update ConversationPanel** to render new response type
4. **Add corresponding Rust response variant** (aof-conversational/src/types.rs)
5. **Update orchestrator logic** to return new variant

**Example: Adding a "preview" mode**

```typescript
// types/conversation.ts
export type OrchestratorResponse =
  | { type: 'preview'; files: Record<string, string>; editable: boolean }
  | /* existing types */;

// conversationSlice.ts
builder.addCase(sendMessage.fulfilled, (state, action) => {
  if (action.payload.response.type === 'preview') {
    // Handle preview response
  }
});
```

---

## Performance Considerations

- **Request-response latency:** 100-500ms depending on Claude API response time
- **File persistence:** <1ms for local filesystem writes
- **WebSocket notifications:** <10ms after persistence
- **No polling:** UI updates from Redux state changes triggered by API responses
- **Session TTL:** 30 minutes (configurable in ConversationSessionStore)

---

## Troubleshooting

**Session not found errors:**
- Sessions expire after 30 minutes of inactivity
- Create a new session if expired

**No pending files to confirm:**
- Ensure the orchestrator returned `specialist_result` type
- Check that `pendingFiles` is set in Redux state

**Files not appearing in AgentGrid after confirmation:**
- Config polling interval is 5 seconds
- Wait or reload the page
- Check WebSocket connection status

**Orchestrator errors:**
- Check `aofctl serve` logs for LLM errors
- Verify workspace path is writable
- Ensure LLM model is configured correctly
