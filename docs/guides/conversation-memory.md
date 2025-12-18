# Conversation Memory System

AOF includes a built-in conversation memory system that maintains context across messages in Slack channels and threads. This allows the AI assistant to remember previous interactions and provide more contextual responses.

## Overview

The conversation memory system stores messages per channel/thread combination and automatically provides previous context to the AI agent when processing new messages.

## How It Works

### Memory Storage

Conversation history is stored in memory using a `DashMap` (concurrent hashmap):

```
channel_id:thread_id -> [ConversationEntry, ConversationEntry, ...]
```

Each entry contains:
- **content**: The message text
- **role**: Either "user" or "assistant"
- **timestamp**: When the message was sent

### Memory Key Strategy

The system uses different keys depending on whether the conversation is in a thread:

- **Channel-only**: `channel_id` (e.g., `C123456`)
- **Thread**: `channel_id:thread_id` (e.g., `C123456:1234567890.123456`)

This means:
- Messages in a Slack thread share memory within that thread
- Messages in the main channel (not in a thread) share memory at the channel level

### Memory Limits

To prevent memory bloat:
- **Max messages**: 20 messages per conversation
- **Context window**: Last 10 messages sent to LLM
- **Message truncation**: Long messages (>500 chars) are truncated in context

### Context Injection

When a new message is received, the system:

1. Retrieves conversation history **BEFORE** storing the current message (to avoid duplicating it)
2. Formats history with clear markers for the LLM:
   ```
   [CONVERSATION HISTORY - Use this to understand references like 'it', 'that', 'the deployment']

   User: list deployments in default namespace

   Assistant: Here are the deployments in default namespace:
   - nginx-deployment (2/2 ready)
   - webex234 (3/3 ready)

   User: can you delete webex234?

   [END CONVERSATION HISTORY]

   [CURRENT USER MESSAGE]
   its a deployment
   ```
3. Stores the user message in memory for future context
4. Sends formatted input to the AI agent
5. Stores the assistant's response in memory

## Technical Implementation

### Data Structures

```rust
/// Conversation memory entry
pub struct ConversationEntry {
    pub content: String,
    pub role: String,  // "user" or "assistant"
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// TriggerHandler stores conversation memory
pub struct TriggerHandler {
    // ... other fields ...
    conversation_memory: Arc<DashMap<String, Vec<ConversationEntry>>>,
}
```

### Key Methods

```rust
// Get conversation key for channel/thread
fn get_conversation_key(channel_id: &str, thread_id: Option<&str>) -> String

// Add message to memory
fn add_to_conversation(&self, channel_id: &str, thread_id: Option<&str>, role: &str, content: &str)

// Get conversation history
fn get_conversation_history(&self, channel_id: &str, thread_id: Option<&str>) -> Vec<ConversationEntry>

// Format history as context for LLM
fn format_conversation_context(&self, channel_id: &str, thread_id: Option<&str>) -> String
```

### Location in Codebase

- **Main implementation**: `crates/aof-triggers/src/handler/mod.rs`
- **Memory is stored in**: `TriggerHandler.conversation_memory`

## Limitations

### Current Limitations

1. **In-Memory Only**: Memory is not persisted to disk
   - Restarting the server clears all conversation history
   - For production, consider implementing Redis/database persistence

2. **No Cross-Server Memory**: Each server instance has its own memory
   - In multi-instance deployments, users may get different contexts

3. **No Memory Expiration**: Old conversations are only evicted when limit is reached
   - Consider implementing TTL-based expiration

### Future Improvements

- [ ] Redis persistence backend
- [ ] SQLite persistence backend
- [ ] Configurable memory limits
- [ ] TTL-based memory expiration
- [ ] Cross-instance memory sharing

## Configuration

Currently, conversation memory is always enabled with default settings. Future versions will support configuration:

```yaml
# Future configuration (not yet implemented)
spec:
  memory:
    enabled: true
    backend: inmemory  # or redis, sqlite
    max_messages: 20
    context_window: 10
    ttl_minutes: 60
```

## Best Practices

### Agent Instructions

When writing agent instructions, **you MUST explicitly tell the agent to use conversation context**. Without this, the LLM may ignore the context:

```yaml
spec:
  instructions: |
    You are a helpful K8s assistant.

    IMPORTANT - Conversation Memory:
    You have access to previous conversation context. When users send follow-up messages like
    "delete it", "can you delete that", "scale it down", or reference "the deployment" without
    specifying which one, ALWAYS use the previous conversation context to understand what
    they're referring to. The context includes recent messages showing what resources were
    discussed.

    When user says "it", "that", "the deployment" etc., check the previous conversation
    context to understand what they mean. NEVER ask the user to clarify when the answer
    is clearly in the conversation history.
```

### Thread Usage

Encourage users to use threads for multi-turn conversations:
- Threads provide cleaner context separation
- Main channel memory can get cluttered with unrelated messages
- Thread memory is isolated and focused

## Troubleshooting

### Bot doesn't remember previous messages

1. **Check if using threads**: Thread memory is separate from channel memory
2. **Server restart**: Memory is cleared on restart
3. **Memory limit reached**: Old messages are evicted after 20 messages

### Enable debug logging

```bash
RUST_LOG=debug aofctl serve --config config.yaml
```

Look for logs like:
```
DEBUG Conversation context length: 1234 chars
DEBUG Processing natural language input for agent k8s-assistant: ...
```

## Example Conversation

```
User: @bot what pods are running in the default namespace?
Bot: Here are the pods in the default namespace:
     - nginx-7b4f9c...
     - redis-master-...

User: @bot can you describe the nginx pod?
Bot: [Uses context to understand "the nginx pod" refers to nginx-7b4f9c...]
     Here are the details for nginx-7b4f9c...:
     ...

User: @bot delete it
Bot: [Uses context to understand "it" refers to nginx-7b4f9c...]
     ⚠️ This action requires approval
     `kubectl delete pod nginx-7b4f9c...`
     React with ✅ to approve or ❌ to deny.
```
