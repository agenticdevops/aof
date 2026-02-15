# Phase 1.5: Onboarding Refinement — Planning Complete

**Date:** 2026-02-15
**Planner:** Claude Code (Haiku 4.5)
**Status:** ✅ PLANNING COMPLETE — Ready for Execution

---

## Overview

Phase 1.5 (Onboarding Refinement) is a focused refinement phase addressing user feedback from early testing. The phase addresses UX friction in the onboarding wizard, implements missing safety/approval infrastructure, and provides production-ready tool discovery and agent templates.

**Key Insight:** Users don't think about "projects" — they think about "channels" (where Xops appears), "tools" (what Xops can do), and "agents" (specialists coordinating). The refined 3-step wizard aligns with user mental models.

---

## Planning Artifacts

Four executable plans created, ready for Claude executor:

| Plan | Objective | Tasks | Scope | Wave |
|------|-----------|-------|-------|------|
| **01-PLAN.md** | Redesign wizard from 4→3 steps | 8 | Wizard flow, Redux, form components | 1 |
| **02-PLAN.md** | Local tool auto-discovery + validation | 7 | Backend discovery, frontend integration | 2 |
| **03-PLAN.md** | Specialist bot templates + squad management | 6 | Template definitions, squad UI, API | 3 |
| **04-PLAN.md** | Safety/approval gates + audit logging | 7 | Operation classification, approval workflow, audit logging | 4 |

**Total:** 4 plans, 28 tasks, 4 execution waves, ~1 week duration

---

## Plan Summaries

### Plan 01: Redesign Onboarding Wizard (Wave 1)

**Objective:** Reduce onboarding from 4 steps (Project → Agent → Platforms → Review) to 3 focused steps (Channels → AI Model → Tools → Review) with Xops auto-creation.

**Key Changes:**
- Remove Project setup step (mental model bloat)
- Streamline to: Channels → AI Model → Tools → Review
- Auto-create Xops orchestrator on completion
- Type-safe Redux state management
- Comprehensive form validation

**Files Modified:** 8
- `onboarding.ts` — Complete types for wizard state
- `onboardingSlice.ts` — Redux reducer with async thunks
- `OnboardingWizard.tsx` — Main 3-step container
- `StepChannels.tsx` — Channel selection (Slack/Telegram/Discord)
- `StepAIModel.tsx` — LLM provider + API key
- `StepTools.tsx` — Tool selection (placeholder for auto-discovery)
- `StepReview.tsx` — Xops profile preview + launch

**Verification:**
- [ ] Wizard loads with Step 1 of 4
- [ ] Navigation works (forward/back/submit)
- [ ] Validation blocks invalid advancement
- [ ] Xops created on submit with orchestrator role
- [ ] State persists across refresh
- [ ] 8+ E2E tests passing
- [ ] No regressions

**Context:** Builds on Phase 1 Integration (API client, Redux, testing infrastructure)

---

### Plan 02: Local Tool Auto-Discovery (Wave 2)

**Objective:** Implement backend tool discovery that scans system paths and returns available tools to the frontend without user manual configuration.

**Key Changes:**
- Backend ToolDiscovery service (aof-tools crate)
- Scan common paths: /usr/bin, /usr/local/bin, ~/.local/bin, etc.
- Detect 5+ critical tools (kubectl, terraform, docker, git, aws-cli)
- Version detection with timeout protection
- Frontend hook useToolDiscovery for tool listing
- StepTools component uses auto-discovery

**Files Modified:** 7
- `aof/crates/aof-tools/src/discovery.rs` — ToolDiscovery struct
- `aof/src/server/handlers/tools.rs` — GET /api/tools/discover handler
- `web-app/src/api/config.ts` — configAPI.discoverTools()
- `web-app/src/services/toolDiscovery.ts` — useToolDiscovery hook
- `web-app/src/test/mocks/handlers.ts` — Mock tool discovery endpoint
- `web-app/src/components/onboarding/StepTools.tsx` — Integrated UI
- Tests for backend + frontend discovery

**Verification:**
- [ ] Tool detection finds >=5 critical tools
- [ ] Version detection works (with timeout protection)
- [ ] Backend caching (1-hour validity)
- [ ] Frontend displays tools grouped by category
- [ ] Recommended tools pre-selected
- [ ] User can enable/disable tools
- [ ] pnpm test all passing
- [ ] cargo test all passing

**Context:** Depends on Plan 01 (wizard structure)

---

### Plan 03: Specialist Bot Templates + Squad Management (Wave 3)

**Objective:** Create 3 pre-configured specialist bot templates (K8s Ops, Infrastructure, SRE) that users can select to instantly get a coordinated agent squad.

**Key Changes:**
- 3 specialist templates with agents, personas, skills
- BotTemplateSelector component for template discovery
- SquadCompositionUI for viewing team members
- API endpoints for squad CRUD (create/get/update/delete)
- Redux integration for squad state management
- Configuration dashboard with squad panel

**Templates:**
1. **Kubernetes Ops Squad** — Incident response, deployments, health checks
   - Agents: Incident Commander, Log Analyzer, Metrics Checker, K8s Diagnostician
   - Tools: kubectl, prometheus, loki, docker
   - Success Metric: MTTR < 5 minutes

2. **Infrastructure Automation Squad** — IaC, provisioning, cost optimization
   - Agents: IaC Provisioner, Cost Optimizer, Security Auditor
   - Tools: terraform, aws-cli, vault
   - Success Metric: $10k+ annual savings identified

3. **SRE Observability Squad** — Health monitoring, anomaly detection
   - Agents: Health Monitor, Anomaly Detector, Trend Analyzer
   - Tools: prometheus, kubectl
   - Success Metric: 99.9% uptime, <2 min detection

**Files Modified:** 8
- `agents.ts` — Agent, template, squad types
- `botTemplates.ts` — 3 specialist template definitions
- `BotTemplateSelector.tsx` — Template discovery UI
- `SquadCompositionUI.tsx` — Squad member display
- `SquadCompositionPanel.tsx` — Dashboard integration
- `configSlice.ts` — Squad state management
- `config.ts` — API methods (createFromTemplate, etc.)
- Tests for templates and squad operations

**Verification:**
- [ ] 3 templates available and selectable
- [ ] Template selection creates all agents
- [ ] Squad composition UI shows agents with personas
- [ ] Can add/remove agents from squad
- [ ] Squad persists across daemon restarts
- [ ] All agents properly coordinated with Xops
- [ ] 6+ E2E tests passing

**Context:** Depends on Plan 01 (wizard completion triggers squad options)

---

### Plan 04: Safety/Approval Gates + Audit Logging (Wave 4)

**Objective:** Implement security-aware operation classification and approval workflow with immutable, cryptographically-verified audit logging for compliance.

**Key Changes:**
- Operation classification: Destructive (requires approval), Risky (approval depends on environment), Safe (immediate)
- Approval workflow: Pending → Approved/Rejected → Executed
- Environment-aware auto-approval (auto in dev/staging, require in prod)
- Immutable audit logging with SHA256 chain integrity
- Slack integration for approval requests (future enhancement)
- Web UI for approvals and audit trail

**Architecture:**

```
Operation Execution
    ↓
Classify (Destructive/Risky/Safe)
    ↓
Needs Approval?
    ├─ Yes → Create ApprovalRequest
    │         Send to Slack/Web UI
    │         Wait for decision
    │         Log approval decision to AuditLog
    │         Execute if approved
    └─ No → Execute immediately
            Log to AuditLog
```

**Audit Log Properties:**
- Immutable (append-only, no modifications)
- Cryptographic chain (SHA256 hash includes previous hash)
- Integrity verifiable (can detect tampering)
- Searchable (by operator, time, operation, result)
- Compliance-grade (legal hold, forensic analysis)

**Files Modified:** 8
- `aof-core/src/approval.rs` — Operation classification, types
- `aof/src/audit/audit_logger.rs` — AuditLogger with file persistence
- `aof/src/server/handlers/approval.rs` — Approval API endpoints
- `ApprovalWorkflow.tsx` — Web UI for approvals
- `AuditTrail.tsx` — Read-only audit log viewer
- `auditSlice.ts` — Redux state for approvals/audit
- `config.ts` — API methods for approvals
- Tests for approval workflow

**Verification:**
- [ ] Operations classified correctly
- [ ] Approval required for destructive ops
- [ ] Auto-approval works in dev/staging
- [ ] Audit log immutable (no delete buttons)
- [ ] Hash chain verifies integrity
- [ ] Approvals via web UI functional
- [ ] Slack integration ready (webhook handler)
- [ ] 6+ E2E tests passing

**Context:** Depends on Plan 01/02 (wizard and tools complete first)

---

## Execution Plan

### Wave 1 (Day 1): Wizard Redesign
- Execute Plan 01 (8 tasks)
- Expected: 2-3 hours
- Blocker: None
- Verification: Wizard works, user can launch Xops in <3 minutes

### Wave 2 (Day 2): Tool Discovery
- Execute Plan 02 (7 tasks)
- Expected: 2-3 hours
- Blocker: Plan 01 must complete (step integration)
- Verification: Tools auto-discovered, UI displays them

### Wave 3 (Day 3-4): Specialist Bots
- Execute Plan 03 (6 tasks)
- Expected: 4-5 hours
- Blocker: Plans 01, 02 must complete
- Verification: 3 templates available, squads can be created

### Wave 4 (Day 5): Safety & Approvals
- Execute Plan 04 (7 tasks)
- Expected: 4-5 hours
- Blocker: Plans 01, 02 must complete
- Verification: Approvals functional, audit log immutable

**Total Estimated Duration:** 5-7 days (1 week)

---

## Must-Haves (Goal-Backward Verification)

### Truth: "User can complete onboarding and launch Xops in <3 minutes"

**Observable behaviors required:**
- ✅ Wizard has 3 steps (Channels → Model → Tools)
- ✅ Each step requires <30 seconds to fill
- ✅ Channels step has 1+ options selected
- ✅ Model has API key entered
- ✅ Tools auto-discovered and visible
- ✅ Launch button creates Xops immediately
- ✅ User sees success confirmation
- ✅ Xops ready to receive messages

**Artifacts required:**
- ✅ OnboardingWizard.tsx with 3-step container
- ✅ StepChannels, StepAIModel, StepTools, StepReview components
- ✅ onboardingSlice with Redux state
- ✅ configAPI methods for creation
- ✅ Backend endpoints for agent creation

**Key links:**
- Wizard → Redux → API → Backend agent creation
- Tools API → Frontend auto-discovery → UI display

---

### Truth: "All critical tools are discoverable without manual setup"

**Observable behaviors:**
- ✅ StepTools shows kubectl, terraform, docker, git, aws-cli when available
- ✅ Tools show version numbers
- ✅ Tools show file paths
- ✅ Recommended tools are pre-selected
- ✅ User can toggle tools on/off

**Artifacts:**
- ✅ ToolDiscovery backend service
- ✅ GET /api/tools/discover endpoint
- ✅ useToolDiscovery React hook
- ✅ StepTools component with UI

**Key links:**
- Backend scanner → API endpoint → Frontend hook → Component display

---

### Truth: "Users can select pre-built agent squads for common scenarios"

**Observable behaviors:**
- ✅ 3 templates visible in selector (K8s, Infrastructure, SRE)
- ✅ Each template shows agents and skills
- ✅ Clicking "Create Squad" creates all agents
- ✅ Squad composition shows all team members
- ✅ Agents visible with personas and roles

**Artifacts:**
- ✅ BotTemplate type and 3 template instances
- ✅ BotTemplateSelector component
- ✅ SquadCompositionUI component
- ✅ API methods for squad creation

**Key links:**
- Templates → Selector → API → Redux → Squad UI

---

### Truth: "Dangerous operations require approval and are immutably logged"

**Observable behaviors:**
- ✅ Delete operations blocked until approved
- ✅ Approval request shows operation details
- ✅ Approve/reject buttons functional
- ✅ Audit log shows all operations
- ✅ Audit log is read-only (no delete)
- ✅ Logs have cryptographic hash chain

**Artifacts:**
- ✅ OperationCategory enum
- ✅ AuditLogger with file persistence
- ✅ ApprovalRequest types
- ✅ Approval API endpoints
- ✅ ApprovalWorkflow and AuditTrail components

**Key links:**
- Operation execution → Classification → Approval gate → AuditLogger

---

## Acceptance Criteria

**Phase 1.5 is COMPLETE when:**

1. ✅ **Wizard redesigned:** 3-step flow (Channels → Model → Tools) working
2. ✅ **Xops auto-created:** Orchestrator agent with pre-defined persona
3. ✅ **Tools auto-discovered:** >=5 critical tools detected and displayed
4. ✅ **Specialist templates:** 3 templates (K8s, Infrastructure, SRE) available
5. ✅ **Squad composition:** Users can view and manage agent teams
6. ✅ **Approval workflow:** Destructive ops require approval, immutably logged
7. ✅ **Audit trail:** Searchable, cryptographically-verified, read-only
8. ✅ **Integration complete:** All components wired into dashboard
9. ✅ **Tests passing:** 28+ E2E tests, 0 regressions
10. ✅ **Type-safe:** 0 TypeScript errors, full type coverage

---

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|-----------|
| **API endpoint missing** | Blocker for frontend | Low | Implement stub endpoints if needed |
| **Tool discovery fails on some systems** | Graceful degradation needed | Medium | Allow empty tool list, error message |
| **Approval workflow too complex** | UX friction | Low | Keep Web UI simple, defer Slack to Phase 2 |
| **Audit logging performance** | Slow operations | Low | Async logging, background writes |
| **Regression in wizard** | Phase 1 broken | Low | Comprehensive test suite, no changes to Phase 1 code |

---

## Success Metrics

**Execution Success:**
- 28 tasks completed (100%)
- 4 plans executed (100%)
- 28+ E2E tests passing (100%)
- 0 TypeScript errors
- 0 regressions in Phase 1

**User Success:**
- Time to launch Xops: <3 minutes
- Tool discovery: >=5 tools found automatically
- Template adoption: >=1 template used per session
- Approval understanding: Clear in UI, actionable

**Quality:**
- Code coverage: >80% for new code
- Accessibility: WCAG AA compliant
- Performance: Wizard <1s per step, discovery <3s
- Documentation: All components documented

---

## File Organization

```
.planning/phases/01-onboarding-refinement/
├── PHASE-BRIEF.md                 ← User feedback and scope
├── RESEARCH.md                    ← Detailed research (1400+ lines)
├── RESEARCH-SUMMARY.md            ← Executive summary
├── 01-PLAN.md                     ← Wizard redesign (8 tasks)
├── 02-PLAN.md                     ← Tool discovery (7 tasks)
├── 03-PLAN.md                     ← Specialist templates (6 tasks)
├── 04-PLAN.md                     ← Approval/audit (7 tasks)
├── PLANNING-SUMMARY.md            ← This file
└── (execution summaries after completion)
    ├── 01-SUMMARY.md
    ├── 02-SUMMARY.md
    ├── 03-SUMMARY.md
    └── 04-SUMMARY.md
```

---

## Handoff to Executor

**Everything ready:**
- ✅ 4 PLAN.md files with detailed task breakdowns
- ✅ Type specifications and data structures defined
- ✅ API contracts specified
- ✅ Component interfaces documented
- ✅ Mock data examples provided
- ✅ Test scenarios outlined
- ✅ Error handling requirements specified

**To execute, run:**
```bash
/gsd:execute-phase 01-onboarding-refinement --plan 01
# Then 02, 03, 04 in sequence
```

**Expected output after execution:**
- 4 execution summaries (SUMMARY.md files)
- 28 git commits (1 per task)
- 28+ E2E tests passing
- 50+ files created/modified
- Phase 1.5 complete and ready for Phase 2

---

## Context for Next Phases

**Phase 2 (Mission Control UI):**
- Uses Xops created by this phase
- Uses squad configuration from specialist templates
- Uses approval workflow for mission operations
- Consumes audit log for activity feed

**Phase 3 (Real Ops Capabilities):**
- Uses tools discovered by this phase
- Uses approval gates for actual operations
- Logs real incidents to audit trail
- Runs specialist agents from templates

**Enterprise Extensions (Future):**
- Slack integration for approvals (current is web UI only)
- Advanced approval policies (multi-party approval)
- Audit compliance reporting (SOC 2, HIPAA)
- Tool marketplace and custom Rust SDK

---

**Planning completed:** 2026-02-15T20:15Z
**Ready for execution:** ✅ YES
**Estimated execution time:** 5-7 days
**Estimated total effort:** ~35-45 hours Claude execution time
