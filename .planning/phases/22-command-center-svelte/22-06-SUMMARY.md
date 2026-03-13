---
phase: 22-command-center-svelte
plan: "06"
subsystem: command-center-approvals
tags:
  - svelte5
  - approval-queue
  - websocket
  - optimistic-ui
  - human-in-the-loop
dependency_graph:
  requires:
    - 22-03  # API client + types (ApprovalRequest, api.approvals.*)
    - websocket-store  # wsEvents for real-time updates
  provides:
    - /approvals page with live approval queue
    - approval-card component
    - approvals reactive store
  affects:
    - sidebar (already linked /approvals route)
tech_stack:
  added: []
  patterns:
    - Svelte 5 runes ($state, $derived, $props)
    - Optimistic UI with rollback on failure
    - WebSocket event subscription pattern (matches agents store)
    - Writable + derived store composition
key_files:
  created:
    - apps/command-center/src/lib/stores/approvals.ts
    - apps/command-center/src/lib/components/approval-card.svelte
    - apps/command-center/src/routes/approvals/+page.svelte
  modified: []
decisions:
  - "Optimistic UI with rollback: immediately update status on approve/deny, revert on API failure"
  - "WebSocket approval_requested: add new pending request at top of list if id not already present"
  - "History section collapsed by default to keep pending requests prominent"
  - "Deny flow uses inline input (not modal) for minimal friction in high-urgency context"
metrics:
  duration_seconds: 159
  completed_date: "2026-03-13"
  tasks_completed: 2
  tasks_total: 2
  files_created: 3
  files_modified: 0
---

# Phase 22 Plan 06: Approval Queue — Summary

**One-liner:** Human-in-the-loop approval queue with optimistic approve/deny actions, WebSocket real-time updates, and urgency-highlighted pending cards.

---

## What Was Built

### Task 1: Approval Store (`approvals.ts`)

Reactive Svelte store module for the approval queue:

- **Stores:** `approvals` (full list), `approvalsLoading`, `approvalsError`
- **Derived:** `pendingApprovals` (status === 'pending'), `decidedApprovals` (all others)
- **`loadApprovals()`** — fetches from `GET /api/v1/approvals`, sets store
- **`approveRequest(id)`** — optimistic: immediately sets status='approved', calls `POST /api/v1/approvals/:id/approve`; reverts on failure
- **`denyRequest(id, reason?)`** — optimistic: immediately sets status='denied', calls `POST /api/v1/approvals/:id/deny`; reverts on failure
- **WebSocket subscription:**
  - `approval_requested` → prepend new pending request (deduplicated by id)
  - `approval_decided` → update matched request status in-place

### Task 2: Approval Card + Queue Page

**`approval-card.svelte`** — Individual request card:
- Header: Bot icon + agent name (bold) + status badge (amber=pending, green=approved, red=denied, gray=expired)
- Body: Action description in highlighted box
- Meta: relative time ("5 min ago"), run_id link to trace view, decided_by
- Footer (pending only): Approve button (green, CheckCircle) + Deny button (red/outlined, XCircle)
- Deny flow: inline text input for optional reason + Confirm/Cancel buttons
- Decided state: outcome icon + decided_at timestamp
- Visual urgency: pulsing amber top border on pending cards

**`approvals/+page.svelte`** — Full approval queue at `/approvals`:
- Header: ShieldCheck icon + "Approval Queue (N pending)" + Refresh button
- Toast notifications (3s auto-dismiss) for approve/deny outcomes
- Error banner with retry
- Filter bar: All / Pending / Approved / Denied
- **Pending section** (prominent, top): grid layout with enter animation on new cards
- **All-clear empty state**: green checkmark, "No pending approvals — all clear"
- **History section** (collapsed by default): decided approvals sorted by decided_at desc, toggle button
- Loading skeletons while fetching

---

## Deviations from Plan

None — plan executed exactly as written.

---

## Verification

- `npm run check` — 0 errors, 3 warnings (pre-existing in agent-form.svelte, unrelated)
- `npm run build` — succeeded, approvals page included in build output
- All required exports present: `approvals`, `loadApprovals`, `approveRequest`, `denyRequest`
- All files meet minimum line requirements: approvals.ts (138 lines), approval-card.svelte (191 lines), +page.svelte (267 lines)

---

## Commits

| Hash    | Message |
|---------|---------|
| 363c722 | feat(22-06): add approval store with WebSocket reactivity and optimistic updates |
| d3b0694 | feat(22-06): add approval queue page and approval card component |

---

## Self-Check: PASSED

- FOUND: apps/command-center/src/lib/stores/approvals.ts
- FOUND: apps/command-center/src/lib/components/approval-card.svelte
- FOUND: apps/command-center/src/routes/approvals/+page.svelte
- FOUND commit: 363c722
- FOUND commit: d3b0694
