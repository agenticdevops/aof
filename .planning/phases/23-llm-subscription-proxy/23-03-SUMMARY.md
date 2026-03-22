---
phase: 23-llm-subscription-proxy
plan: 03
subsystem: llm
tags: [subscription, oauth, bearer-auth, provider-factory, model-trait, anthropic, openai, google]

requires:
  - phase: 23-llm-subscription-proxy
    plan: 01
    provides: "ProviderMode enum, OAuthConfig, auth foundation types"

provides:
  - "AnthropicSubscriptionProvider: implements Model trait with Bearer auth instead of x-api-key"
  - "OpenAISubscriptionProvider: thin wrapper over standard OpenAI provider (already Bearer auth)"
  - "GoogleSubscriptionProvider: implements Model trait with Bearer auth instead of ?key= URL param"
  - "ProviderFactory routing: extra['provider_mode'] = 'subscription' selects subscription adapters"
  - "Backward compatibility: all existing API key routing unchanged"

affects:
  - 23-04
  - 23-05
  - 23-06

tech-stack:
  added: []
  patterns:
    - "ProviderMode routing via ModelConfig.extra['provider_mode'] JSON field"
    - "Subscription adapters self-contained: each file includes its own API type definitions"
    - "401 handling: descriptive error messages directing users to agentix auth login <provider>"
    - "OpenAI subscription delegates to standard provider (OAuth token = Bearer auth = same format)"

key-files:
  created:
    - "crates/agentix-llm/src/provider/subscription/mod.rs"
    - "crates/agentix-llm/src/provider/subscription/anthropic_sub.rs"
    - "crates/agentix-llm/src/provider/subscription/openai_sub.rs"
    - "crates/agentix-llm/src/provider/subscription/google_sub.rs"
    - "crates/agentix-llm/tests/subscription_provider_test.rs"
  modified:
    - "crates/agentix-llm/src/provider.rs"
    - "crates/agentix-runtime/src/gateway/api.rs"
    - "crates/agentix/src/commands/onboard.rs"

key-decisions:
  - "Each subscription adapter is self-contained (private API types) rather than cross-module sharing — prevents coupling and allows each adapter to evolve independently"
  - "OpenAI subscription is a thin wrapper over standard OpenAI provider since both use Bearer auth — no duplicate HTTP logic needed"
  - "ProviderFactory routing via ModelConfig.extra['provider_mode'] avoids adding a new parameter to the create() signature — backward compatible without signature change"
  - "Token refresh is out-of-scope for adapters — CallerService (plan 02) ensures valid token before factory call; 401 errors propagate as descriptive AofError::Model"

requirements-completed: [SUB-04, SUB-03]

duration: 9min
completed: 2026-03-22
---

# Phase 23 Plan 03: Subscription Provider Adapters Summary

**Three OAuth Bearer token LLM provider adapters (Anthropic, OpenAI, Google) + ProviderFactory routing — agents run via subscription or API key with zero code changes**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-22T05:11:44Z
- **Completed:** 2026-03-22T05:20:44Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Created `subscription/` module with three provider adapters, each implementing the `Model` trait:
  - `AnthropicSubscriptionProvider`: replaces `x-api-key` header with `Authorization: Bearer` at the standard Messages API endpoint
  - `OpenAISubscriptionProvider`: thin wrapper over `OpenAIProvider` since OpenAI already uses Bearer auth — just intercepts 401 with clearer error message
  - `GoogleSubscriptionProvider`: replaces `?key=API_KEY` URL param with `Authorization: Bearer` header to the Generative Language API
- Extended `ProviderFactory::create()` to check `ModelConfig.extra["provider_mode"]` — when set to `"subscription"` routes to the appropriate subscription adapter; all other modes fall through to existing unchanged API key routing
- 28 tests total (15 unit tests in adapter files + 13 integration tests in `subscription_provider_test.rs`)

## Task Commits

1. **Task 1: Subscription provider adapters** - `f4680ab` (feat)
2. **Task 2: ProviderFactory routing + integration tests** - `6e22934` (feat)

## Files Created/Modified

- `crates/agentix-llm/src/provider/subscription/mod.rs` — Module re-exports and documentation
- `crates/agentix-llm/src/provider/subscription/anthropic_sub.rs` — Anthropic Bearer auth adapter (145 lines + 40 lines tests)
- `crates/agentix-llm/src/provider/subscription/openai_sub.rs` — OpenAI OAuth wrapper (100 lines + 35 lines tests)
- `crates/agentix-llm/src/provider/subscription/google_sub.rs` — Google Bearer auth adapter (370 lines + 50 lines tests)
- `crates/agentix-llm/src/provider.rs` — Extended ProviderFactory with subscription routing
- `crates/agentix-llm/tests/subscription_provider_test.rs` — 13 integration tests

## Decisions Made

- **Self-contained adapters:** Each subscription adapter file declares its own private API types (mirrored from the standard provider files). This prevents coupling — each adapter can evolve independently without affecting the other.
- **OpenAI thin wrapper:** Since `OpenAIProvider` already uses `Authorization: Bearer {token}`, the subscription variant simply passes the OAuth token as `api_key` and wraps to intercept 401 errors with clearer messages. No HTTP duplication.
- **Factory routing via extra["provider_mode"]:** Rather than adding a new parameter to `ProviderFactory::create()` or `ModelConfig`, we read `provider_mode` from the existing `extra: HashMap<String, serde_json::Value>` field. This preserves backward compatibility — configs without this key continue to work unchanged.
- **Token refresh out of scope:** Adapters do not handle OAuth token refresh. The `AuthService` (plan 02) is responsible for providing a valid token before calling the factory. Mid-run expiry returns a descriptive error directing users to `agentix auth login <provider>`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed ProviderConfig struct literal incompatibility**
- **Found during:** Overall workspace verification after Task 2
- **Issue:** `ProviderConfig` was extended in plan 01 with `mode` and `oauth` fields. Two files in `agentix-runtime` and `agentix` used exhaustive struct literals without these new fields, causing E0063 compilation errors.
- **Files modified:** `crates/agentix-runtime/src/gateway/api.rs`, `crates/agentix/src/commands/onboard.rs`
- **Fix:** Added `mode: None, oauth: None` to the five struct literal initializers
- **Committed in:** 6e22934 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 pre-existing bug from plan 01)
**Impact on plan:** Fix required for `cargo check` to succeed. No scope change.

## Test Results

```
agentix-llm lib tests:     23 passed, 0 failed
anthropic integration:     11 passed, 0 failed
bedrock integration:        0 passed (skipped - feature not enabled)
google integration:        12 passed, 0 failed
openai integration:        13 passed, 0 failed
provider integration:       9 passed, 0 failed
subscription integration:  13 passed, 0 failed
```

Total: 81 passing, 0 failing

## Self-Check

Files created:
- [x] `crates/agentix-llm/src/provider/subscription/mod.rs`
- [x] `crates/agentix-llm/src/provider/subscription/anthropic_sub.rs`
- [x] `crates/agentix-llm/src/provider/subscription/openai_sub.rs`
- [x] `crates/agentix-llm/src/provider/subscription/google_sub.rs`
- [x] `crates/agentix-llm/tests/subscription_provider_test.rs`

Commits:
- [x] f4680ab: feat(23-03): add subscription provider adapters
- [x] 6e22934: feat(23-03): extend ProviderFactory with subscription routing + integration tests

---
*Phase: 23-llm-subscription-proxy*
*Completed: 2026-03-22*
