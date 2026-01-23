# TUI Enhancement Plan

## Overview

Enhance the AOF agentic console TUI to provide a sophisticated, LazyGit-inspired experience with rich agent activity logging, cancellation support, and conversation persistence.

## Current State

The current TUI (`crates/aofctl/src/commands/run.rs`) provides:
- Two-column layout (60% chat, 40% system log + context usage)
- Chat history with user/assistant/error messages
- Token usage gauge
- Spinner animation during execution
- Basic keyboard navigation (scroll, enter, ctrl+c)
- Tracing log capture (but system log panel is mostly empty)

## Enhancements

### 1. Agent Activity Logging (System Log Panel)

**Goal**: Make the System Log panel show real-time agent activity.

**Activity Types to Log**:
- 🧠 **Thinking**: "Analyzing user request..."
- 🔍 **Analyzing**: "Examining context from previous messages..."
- 🛠️ **Tool Use**: "Executing: kubectl get pods -n default"
- ⏳ **Waiting**: "Waiting for LLM response..."
- ✓ **Complete**: "Tool execution completed in 234ms"
- ⚠️ **Warning**: "Approaching token limit (85%)"
- 📊 **Metrics**: "Input: 150 tokens, Output: 420 tokens"

**Implementation**:
- Create `AgentActivityLog` struct with activity types
- Add activity channel from executor to TUI
- Emit activities at key execution points:
  - Before LLM call
  - During tool discovery
  - Before/after each tool execution
  - On response parsing
  - On error conditions

### 2. Escape Key Cancellation

**Goal**: Allow users to stop a running agent with Escape key.

**Implementation**:
- Add `CancellationToken` from `tokio_util`
- Pass token to executor
- Check for Escape key during execution loop
- Trigger graceful cancellation
- Show "Cancelled by user" in chat

**UI Changes**:
- Show "Press ESC to cancel" in footer during execution
- Show cancellation status in system log

### 3. Conversation Persistence

**Goal**: Save conversation history for later resumption.

**File Format**: JSON (human-readable, easy to inspect)

**Session File Structure**:
```json
{
  "session_id": "uuid",
  "agent_name": "k8s-helper",
  "model": "google:gemini-2.5-flash",
  "created_at": "2024-01-23T12:00:00Z",
  "updated_at": "2024-01-23T12:30:00Z",
  "messages": [
    {"role": "user", "content": "list pods", "timestamp": "..."},
    {"role": "assistant", "content": "...", "timestamp": "...", "tokens": {"in": 50, "out": 120}}
  ],
  "token_usage": {"input": 500, "output": 1200},
  "activity_log": [...]
}
```

**Storage Location**: `~/.aof/sessions/<agent-name>/<session-id>.json`

**Commands**:
- `aofctl run agent -f agent.yaml --resume` - Resume latest session
- `aofctl run agent -f agent.yaml --resume <session-id>` - Resume specific session
- `aofctl sessions list` - List saved sessions
- `aofctl sessions delete <session-id>` - Delete session

### 4. LazyGit-Inspired UI Enhancements

**Visual Improvements**:
- Better border styling (rounded corners option)
- Color-coded activity types in system log
- Keyboard shortcuts panel (toggle with `?`)
- Status indicators with icons
- Progress bars for long operations
- Breadcrumb navigation

**New Panels/Features**:
- Help overlay (press `?`)
- Activity filter (press `f` to filter log types)
- Compact mode toggle (press `c`)
- Session info panel

**Color Scheme** (keeping minimalist but adding semantic colors):
- White: Primary text
- Gray: Secondary/dimmed
- Cyan: Thinking/analyzing activities
- Yellow: Tool execution
- Green: Success/complete
- Red: Errors
- Magenta: System messages

### 5. Enhanced Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Esc` | Cancel running agent / Close popup |
| `Ctrl+C` | Quit application |
| `?` | Toggle help panel |
| `f` | Toggle activity filter |
| `c` | Toggle compact mode |
| `Shift+↑/↓` | Scroll chat |
| `PageUp/Down` | Scroll chat (5 lines) |
| `Tab` | Switch focus between panels |
| `Ctrl+S` | Save session manually |
| `Ctrl+L` | Clear chat (new session) |
| `/` | Search in history |

### 6. Footer Enhancements

**Current Footer**:
```
✓ Completed │ 5 messages │ Model: google:gemini-2.5-flash │ Tools: shell, kubectl │ Last: 234ms
```

**Enhanced Footer** (context-aware):
```
[Idle] ✓ 5 msgs │ google:gemini-2.5-flash │ 3 tools │ IN: 500 OUT: 1.2K (1.7K total) │ ?:help ESC:cancel
```

```
[Running] ◐ 2.3s │ Executing tool: kubectl │ ESC to cancel
```

## Implementation Order

1. **Phase 1: Activity Logging** (Priority: High)
   - Add activity events to executor
   - Display in system log panel
   - Color-code by activity type

2. **Phase 2: Cancellation** (Priority: High)
   - Add CancellationToken support
   - Handle Escape key
   - Graceful cleanup

3. **Phase 3: Session Persistence** (Priority: Medium)
   - Create session file format
   - Auto-save on exit
   - Resume from file

4. **Phase 4: UI Polish** (Priority: Medium)
   - Help overlay
   - Enhanced keybindings
   - Better styling

5. **Phase 5: Advanced Features** (Priority: Low)
   - Search in history
   - Activity filters
   - Compact mode

## Files to Modify

- `crates/aofctl/src/commands/run.rs` - Main TUI implementation
- `crates/aof-runtime/src/executor/mod.rs` - Add activity events
- `crates/aof-runtime/src/executor/agent_executor.rs` - Emit activities
- `crates/aofctl/src/cli.rs` - Add --resume flag
- `crates/aofctl/src/commands/mod.rs` - Add sessions command

## New Files to Create

- `crates/aofctl/src/session.rs` - Session management
- `crates/aof-core/src/activity.rs` - Activity event types
- `docs/guides/tui-guide.md` - TUI documentation
