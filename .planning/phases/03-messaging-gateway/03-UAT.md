# Phase 3 UAT (User Acceptance Testing)

**Phase:** 03 - Messaging Gateway
**Date Started:** 2026-02-13
**Tester:** User

---

## Test Approach

Conversational testing of Phase 3 deliverables. Each test validates one observable behavior from the planning and execution summaries. Tests cover:

1. **Core Infrastructure** (03-01) - Hub, adapters, event translation, rate limiting
2. **Platform Adapters** (03-02) - Slack, Discord, Telegram implementations
3. **Squad & Integration** (03-03) - Squad broadcast, YAML config, aofctl integration

---

## Test Cases

### CORE INFRASTRUCTURE TESTS

#### TEST 1: Gateway Hub initialization and adapter registration
**Precondition:** aof-gateway crate compiles and tests pass
**Expected Behavior:** Gateway hub can register adapters and maintain adapter registry

```rust
// From 03-01: GatewayHub initializes with:
// - Session ID (UUID)
// - Empty adapter registry (HashMap)
// - Rate limiter registry (HashMap)
// - Event broadcast channel
```

**Acceptance:** Hub can be created, adapters added/removed, and queried
**Evidence:** Integration test in 03-01-SUMMARY lines 134-135

**Status:** ⬜ Pending
**Result:**

---

#### TEST 2: InboundMessage → CoordinationEvent translation preserves all message details
**Precondition:** Translation module compiles
**Expected Behavior:** Platform messages translate to CoordinationEvent with metadata intact

```rust
// From 03-01: Event translation layer maps:
// InboundMessage { platform, sender, content, thread, attachments, metadata }
//      ↓
// CoordinationEvent {
//   agent_id: format!("gateway-{:?}", platform),
//   event_type: ActivityEvent::Info {
//     metadata: { "content": markdown, "user": sender, ...}
//   }
// }
```

**Acceptance:** Message details not lost in translation; metadata preserved
**Evidence:** Translation tests in 03-01-SUMMARY lines 126-127

**Status:** ⬜ Pending
**Result:**

---

#### TEST 3: Rate limiting (GCRA token bucket) enforces per-platform quotas without blocking others
**Precondition:** RateLimiter module compiles
**Expected Behavior:** Rate limiters enforce async-ready quota (1/10/30 req/sec per platform)

```rust
// From 03-01: Each platform gets rate limiter:
// - Slack: 1 req/sec, burst 5
// - Discord: 10 req/sec, burst 20
// - Telegram: 30 msg/sec, burst 50
// acquire().await blocks until token available
// check() returns Err immediately if exhausted
```

**Acceptance:** Quotas enforced correctly; Slack limited to 1/sec while Discord handles 10/sec
**Evidence:** Rate limiter tests in 03-01-SUMMARY lines 127-128

**Status:** ⬜ Pending
**Result:**

---

#### TEST 4: YAML config loads, validates, and substitutes environment variables
**Precondition:** Config.rs compiles; .env file with test values exists
**Expected Behavior:** Gateway config loads from YAML, validates schema, replaces ${VAR} with env values

```yaml
# From 03-03: Config format (apiVersion: aof.dev/v1, kind: Gateway)
# With environment variable substitution:
# SLACK_TOKEN=xoxb-... DISCORD_TOKEN=...
#   ↓
# spec.adapters[0].config.token: "${SLACK_TOKEN}" → "xoxb-..."
```

**Acceptance:** Config loads, env vars substituted, validation catches missing vars (all at once, not one at a time)
**Evidence:** Config tests in 03-01-SUMMARY lines 128-129; 03-03-SUMMARY lines 122-140

**Status:** ⬜ Pending
**Result:**

---

### PLATFORM ADAPTER TESTS

#### TEST 5: Slack adapter validates token and sends messages via HTTP
**Precondition:** Slack adapter module compiles
**Expected Behavior:** Adapter validates Slack token on start; can send messages via chat.postMessage API

```rust
// From 03-02: Slack adapter (282 lines)
// - Token validation: POST /api/auth.test → validates bearer token
// - Message sending: POST /api/chat.postMessage with Block Kit JSON
// - Rate limiting: 1 req/sec enforced
// - Threading: thread_ts support for reply chains
// - Stale filtering: messages >5 min old dropped
```

**Acceptance:** Auth validation works (or fails gracefully with helpful error); Message sends work
**Evidence:** Slack adapter tests in 03-02-SUMMARY lines 61-62

**Status:** ⬜ Pending
**Result:**

---

#### TEST 6: Discord adapter validates token and sends rich messages (embeds) via HTTP
**Precondition:** Discord adapter module compiles
**Expected Behavior:** Adapter validates Discord token on start; sends messages with embeds

```rust
// From 03-02: Discord adapter (312 lines)
// - Token validation: GET /api/v10/users/@me with Bot token
// - Message sending: POST /channels/{id}/messages with embeds
// - Rate limiting: 10 req/sec enforced
// - Markdown translation: Discord embeds with blurple color (0x5865F2)
// - Long response splitting: >5,500 char responses split into multiple
// - Character limits: Embed description max 4,096 chars
```

**Acceptance:** Auth validation works; Messages send with rich formatting; Long messages split correctly
**Evidence:** Discord adapter tests in 03-02-SUMMARY lines 63-71

**Status:** ⬜ Pending
**Result:**

---

#### TEST 7: Telegram adapter validates token and sends messages via long polling infrastructure
**Precondition:** Telegram adapter module compiles
**Expected Behavior:** Adapter validates Telegram token on start; sends messages with MarkdownV2

```rust
// From 03-02: Telegram adapter (287 lines)
// - Token validation: GET /bot{token}/getMe
// - Message sending: POST /sendMessage with MarkdownV2 formatting
// - Rate limiting: 30 msg/sec enforced
// - Markdown escaping: 18 special characters escaped for MarkdownV2
// - Threading: reply_to_message_id support for reply chains
// - Long polling infrastructure in place (TODO: full getUpdates loop)
```

**Acceptance:** Auth validation works; Messages send with proper MarkdownV2 escaping
**Evidence:** Telegram adapter tests in 03-02-SUMMARY lines 72-80

**Status:** ⬜ Pending
**Result:**

---

#### TEST 8: Retry logic with exponential backoff + Retry-After extraction handles 429 errors gracefully
**Precondition:** Retry module compiles
**Expected Behavior:** Failed requests retry with exponential backoff + jitter; extracts Retry-After header

```rust
// From 03-02: Retry logic (95 lines)
// - Exponential backoff: Base delay × 2^attempt
// - Jitter: Random 0-1000ms added
// - Retry-After extraction: Parses header from error responses
// - Error classification: Retryable (429, network, timeout) vs non-retryable
// - Max retries: 3 attempts by default
// - Logging: Structured warnings with attempt count and delay
```

**Acceptance:** Retryable errors (429) retry up to 3 times with increasing delays; non-retryable errors fail immediately
**Evidence:** Retry logic tests in 03-02-SUMMARY lines 92-93

**Status:** ⬜ Pending
**Result:**

---

### SQUAD & INTEGRATION TESTS

#### TEST 9: Squad configuration defines agents, channels, and membership correctly
**Precondition:** Config compiles; squad config in YAML valid
**Expected Behavior:** Squad schema stores name, description, agents, and per-platform channel IDs

```rust
// From 03-03: Squad schema
// - SquadConfig { name, description, agents, channels }
// - SquadChannels { slack_channel_id, discord_channel_id, telegram_chat_id }
// - Validation: Squad names unique; at least one channel per squad
// - Helpers: get_squad(), get_squad_agents(), get_squad_channels()
```

**Acceptance:** Squad defined in YAML; names validated unique; channel lookups work
**Evidence:** Squad config tests in 03-03-SUMMARY lines 57-75

**Status:** ⬜ Pending
**Result:**

---

#### TEST 10: Squad broadcast sends message to correct agents/channels (best-effort delivery)
**Precondition:** Broadcast module compiles; hub + squad config initialized
**Expected Behavior:** Broadcast resolves target (AllAgents/Squad/Agents/Channel) → finds agents → sends via adapters

```rust
// From 03-03: Broadcast targets
// - AllAgents: Send to all agents in all squads
// - Squad(name): Send to all agents in named squad
// - Agents(ids): Send to specific agent IDs
// - Channel{platform, channel_id}: Send to specific platform channel
//
// Best-effort: Failed channels logged, don't block others
// Returns: BroadcastResult { sent_count, failed_channels }
```

**Acceptance:** Different broadcast targets resolve correctly; failures don't block successes
**Evidence:** Squad broadcast tests in 03-03-SUMMARY lines 77-96

**Status:** ⬜ Pending
**Result:**

---

#### TEST 11: aofctl serve --gateway-config flag starts gateway with config validation
**Precondition:** aofctl compiles with gateway integration
**Expected Behavior:** CLI flags parse correctly; server starts with gateway if config provided

```bash
# From 03-03: CLI flags (lines 148-159)
aofctl serve --gateway-config gateway.yaml            # Start with gateway
aofctl serve --gateway-config gateway.yaml --debug-gateway  # Enable DEBUG logs
aofctl serve --gateway-config gateway.yaml --validate-config # Validate and exit
aofctl serve --port 8080                              # Works without gateway (backward compatible)
```

**Acceptance:** Flags documented; gateway starts when config provided; validation mode works; backward compatible
**Evidence:** CLI integration in 03-03-SUMMARY lines 168-188

**Status:** ⬜ Pending
**Result:**

---

#### TEST 12: Secrets management: Token masking + environment variable aggregation
**Precondition:** Config module compiles; secrets management methods available
**Expected Behavior:** Missing env vars aggregated into single error; tokens masked in logs

```rust
// From 03-03: Secrets management
// - resolve_env_vars(): Returns all missing vars at once (not just first)
//   Error: "Missing required environment variables: SLACK_TOKEN, DISCORD_TOKEN"
// - sanitize_config_for_logging(): Masks tokens (first 8 chars only)
//   "xoxb-123..." safe to log
// - .env file support: load_config_with_dotenv() for development
```

**Acceptance:** All missing vars shown in single error; tokens masked in logs; .env file works
**Evidence:** Secrets tests in 03-03-SUMMARY lines 122-140

**Status:** ⬜ Pending
**Result:**

---

## Test Summary

| # | Test Case | Status | Result | Notes |
|---|-----------|--------|--------|-------|
| 1 | Hub initialization & adapter registry | ⬜ | | |
| 2 | Event translation (InboundMessage → CoordinationEvent) | ⬜ | | |
| 3 | Rate limiting (GCRA, per-platform quotas) | ⬜ | | |
| 4 | YAML config + env var substitution + validation | ⬜ | | |
| 5 | Slack adapter (token validation + HTTP messaging) | ⬜ | | |
| 6 | Discord adapter (token validation + embed translation) | ⬜ | | |
| 7 | Telegram adapter (token validation + MarkdownV2) | ⬜ | | |
| 8 | Retry logic (exponential backoff + Retry-After) | ⬜ | | |
| 9 | Squad configuration (names, channels, members) | ⬜ | | |
| 10 | Squad broadcast (target resolution + best-effort) | ⬜ | | |
| 11 | aofctl serve --gateway-config integration | ⬜ | | |
| 12 | Secrets management (masking + error aggregation) | ⬜ | | |

---

## Success Criteria

**Phase 3 is ACCEPTED if:**
- ✅ All 12 test cases pass OR have issues traced to root cause and documented
- ✅ No critical issues (security, data loss, crashes)
- ✅ Issues found documented with fix plans ready for `/gsd:execute-phase 3 --gaps-only`

**Phase 3 is REJECTED if:**
- ❌ Unable to start gateway without errors
- ❌ Rate limiting doesn't work (adapters ignore quotas)
- ❌ Messages not routed to agents (translation broken)

---

## Next Steps

After UAT completes:
- **If PASSED:** Update STATE.md, ready for Phase 4 planning
- **If ISSUES FOUND:** Create gap closure plan, execute fixes, re-test
- **Phase 4:** Mission Control UI (Leptos WASM dashboard with real-time event visualization)

---

*UAT Created: 2026-02-13*
*Ready for conversational testing*
