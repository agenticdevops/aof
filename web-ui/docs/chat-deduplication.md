# Chat Message Deduplication & Reconnection Recovery

## Overview

The chat system implements message deduplication and reconnection recovery to ensure reliable message delivery even during network interruptions.

## Features

### 1. Optimistic Message ID Generation

Client-side temporary IDs prevent duplicate message appearance during network round-trip:

```typescript
// Format: temp_{timestamp}_{random}
const optimisticId = `temp_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
```

**Example:** `temp_1771037093_x9k2n4p`

### 2. Message Deduplication

The `chatSlice` dedups messages by ID in the `addMessage` reducer:

```typescript
// Check if message already exists
const exists = state.messages.some((m) => m.id === message.id);
if (exists) return;
```

**Edge case handled:** Temp ID replacement when server confirms message:

```typescript
// Find temp message with same content
const tempIndex = state.messages.findIndex(
  (m) => m.id.startsWith('temp_') && m.content === message.content
);
if (tempIndex !== -1 && !message.id.startsWith('temp_')) {
  // Replace temp with real ID from server
  state.messages[tempIndex] = message;
}
```

### 3. Reconnection Recovery

On WebSocket reconnect, `useChatMessages` fetches missing messages:

```typescript
useEffect(() => {
  if (connected && lastMessageId && messages.length > 0) {
    fetchSince(lastMessageId);
  }
}, [connected, lastMessageId, messages.length, fetchSince]);
```

**API call:** `GET /api/chat/messages?since={lastMessageId}`

Messages sent during disconnect are fetched and deduped automatically.

## Workflow

### Normal Message Flow

1. User types message and clicks Send
2. Optimistic message created with `temp_*` ID
3. Message appears immediately in UI
4. POST /api/chat/messages sent to server
5. Server responds with confirmed message (real ID)
6. Temp message replaced with confirmed message

### Reconnection Flow

1. WebSocket disconnects (network issue)
2. User sends message (stored with temp ID, POST may fail)
3. WebSocket reconnects
4. `useChatMessages` detects reconnect
5. Fetch messages since last known ID: `GET /api/chat/messages?since={lastMessageId}`
6. Server returns messages sent during disconnect
7. Messages deduped and merged into Redux state

### Deduplication Scenarios

**Scenario A: Duplicate server message**
- Message arrives via POST response
- Same message arrives via WebSocket event
- Dedup: Second arrival rejected (ID already exists)

**Scenario B: Reconnect fetch includes existing messages**
- Client has messages 1-10
- Disconnect at message 10
- Messages 11-15 sent during disconnect
- Reconnect fetches messages since ID 10
- Server returns 11-15
- Client dedups: 11-15 added (new IDs)

**Scenario C: Temp ID replacement**
- Client sends message with `temp_123`
- Server responds with real ID `msg_456`
- Dedup finds temp message by content
- Replaces `temp_123` with `msg_456`

## Error Handling

### Failed Message Send

Currently: Optimistic message remains in state (TODO: rollback)

**Future improvement:**
```typescript
catch (err) {
  // Remove optimistic message on failure
  dispatch(removeMessage(optimisticMessage.id));
  // Show error toast
  toast.error('Failed to send message');
}
```

### Failed Reconnect Fetch

Non-fatal: Logs error, does not clear existing messages

```typescript
catch (err) {
  console.error('Failed to fetch messages since reconnect:', err);
  // Messages remain in state, user can retry manually
}
```

## Testing Deduplication

### Manual Test: Simulate Disconnect/Reconnect

1. Open DevTools → Network tab
2. Set throttling to "Offline"
3. Send a message (appears with temp ID)
4. Set throttling to "Online"
5. Verify message appears only once (not duplicated)
6. Check Redux DevTools: temp ID replaced with real ID

### Expected Redux State Changes

**Before send:**
```json
{
  "messages": [],
  "lastMessageId": ""
}
```

**After optimistic update:**
```json
{
  "messages": [
    { "id": "temp_1771037093_x9k2n4p", "content": "Test", ... }
  ],
  "lastMessageId": ""
}
```

**After server confirmation:**
```json
{
  "messages": [
    { "id": "msg_abc123", "content": "Test", ... }
  ],
  "lastMessageId": "msg_abc123"
}
```

## Future Enhancements

1. **Message versioning** for conflict resolution
2. **Rollback failed sends** (remove optimistic message)
3. **Retry mechanism** for failed POST requests
4. **Pagination** for large message histories
5. **Message read receipts** (track last read position)
