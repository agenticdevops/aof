---
phase: 19-security
plan: 01
subsystem: security
tags: [ssrf, capability-policy, security-config, ip-validation]

requires:
  - phase: 18-telemetry
    provides: telemetry types referenced by security audit trail
provides:
  - SsrfGuard for URL validation before outbound HTTP requests
  - SsrfViolation error type for SSRF policy violations
  - SecurityConfig for workspace-level security settings
  - CapabilityPolicy for WASM tool capability enforcement
affects: [19-security, 20-approvals]

tech-stack:
  added: []
  patterns: [ssrf-guard-pattern, security-config-defaults]

key-files:
  created:
    - crates/agentix-core/src/security.rs
    - crates/agentix-core/tests/security_test.rs
    - docs/concepts/security.md
  modified:
    - crates/agentix-core/src/lib.rs

key-decisions:
  - "Used std::net::IpAddr for IP parsing instead of external crate — stdlib sufficient for SSRF checks"
  - "SsrfGuard checks IP ranges statically without DNS resolution — prevents DNS rebinding but requires allow-list for hostnames"
  - "SecurityConfig defaults all protections to true — opt-out rather than opt-in"

patterns-established:
  - "SSRF guard pattern: check_url() returns Result<(), SsrfViolation> — callers handle violations"
  - "Security config pattern: default_true() serde defaults for all boolean security flags"

requirements-completed: [SEC-05, SEC-02]

duration: 8min
completed: 2026-03-13
---

# Plan 19-01: Security Core Types Summary

**SsrfGuard blocking RFC 1918/localhost/cloud-metadata IPs, SecurityConfig with sane defaults, CapabilityPolicy for WASM enforcement**

## Performance

- **Duration:** 8 min
- **Tasks:** 3
- **Files created:** 3
- **Files modified:** 1

## Accomplishments
- SsrfGuard validates URLs against blocked IP ranges (RFC 1918, loopback, link-local)
- SsrfGuard supports configurable allow-list for internal services
- SecurityConfig with all protections enabled by default
- CapabilityPolicy struct for WASM tool sandboxing
- 10 unit tests covering all IP ranges, allow-list overrides, config serde
- Security concepts documentation covering all four Phase 19 pillars

## Task Commits

1. **Task 1+2: Security types TDD** - `7ec5998` (feat)
2. **Task 3: Security concepts docs** - `d0f2191` (docs)

## Files Created/Modified
- `crates/agentix-core/src/security.rs` - SsrfGuard, SsrfViolation, SecurityConfig, CapabilityPolicy
- `crates/agentix-core/tests/security_test.rs` - 10 unit tests for all security types
- `crates/agentix-core/src/lib.rs` - Added security module and re-exports
- `docs/concepts/security.md` - Four-pillar security concepts documentation

## Decisions Made
- Used std::net::IpAddr for IP parsing — stdlib sufficient, no external deps needed
- SsrfGuard performs static IP checks without DNS resolution to prevent DNS rebinding
- All SecurityConfig booleans default to true (opt-out security)

## Deviations from Plan
None - plan executed as specified.

## Issues Encountered
None

## Next Phase Readiness
- Security types ready for runtime integration (Plan 19-04)
- SecurityConfig ready for WorkspaceSpec integration (Plan 19-04)

---
*Plan: 19-01-security*
*Completed: 2026-03-13*
