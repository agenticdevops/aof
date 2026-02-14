# Phase 4: Mission Control UI - Planning Overview

**Phase Status:** Planning Complete
**Research Status:** Complete (04-RESEARCH.md)
**Planning Status:** 4 executable PLAN.md files created
**Timeline:** 4 weeks (28 days) - 4 plans, 1 week per plan (Wave 1 = weeks 1-2, Wave 2 = weeks 3-4)

## Phase Goal

Operators see their agent squad coordinating in real-time through a beautiful web dashboard. UI reflects workspace configuration (not hardcoded).

## Requirements Satisfied (MCUI-01 through MCUI-07)

| Req ID | Description | Plan | Status |
|--------|-------------|------|--------|
| MCUI-01 | Web dashboard with clean UI | 04-01, 04-02 | Specified |
| MCUI-02 | Agent cards (avatar, role, status, personality, skills) | 04-02 | Specified |
| MCUI-03 | Kanban task board (5 lanes: backlog/assigned/in-progress/review/done) | 04-02 | Specified |
| MCUI-04 | Squad chat panel (real-time conversation) | 04-03 | Specified |
| MCUI-05 | Live activity feed (agent actions) | 04-03 | Specified |
| MCUI-06 | Task detail view (description, context, assignee, comments, timeline) | 04-03 | Specified |
| MCUI-07 | Squad overview (visual agent network) | 04-02 | Specified |

## Four Execution Plans

### 04-01: Frontend Setup & WebSocket Integration (Wave 1, ~1 week)

**Goal:** React app scaffolded, connected to Phase 1 WebSocket, receives real-time events

**Key Deliverables:**
- React + Vite project with TypeScript strict mode
- Redux store with eventsSlice (receives CoordinationEvent stream)
- useWebSocket hook with automatic reconnection (exponential backoff)
- useAgentsConfig and useToolsConfig hooks for API data fetching
- Tailwind CSS + shadcn/ui component framework
- Hot module reload (HMR) for development velocity
- Build optimization (<500KB gzipped)

**Files:** 10 tasks, establishes foundation for all subsequent plans

**Success Criteria:**
- `npm run dev` starts at localhost:5173
- WebSocket connects to ws://localhost:8080/ws
- CoordinationEvent stream displays in Redux DevTools
- Configuration APIs reachable, even if returning empty defaults
- Hot reload preserves Redux state and WebSocket connection

---

### 04-02: Agent Visualization & Kanban Board (Wave 1, ~1 week)

**Goal:** Agent cards render dynamically, kanban board with drag-and-drop, optimistic updates with version-based conflict resolution

**Key Deliverables:**
- AgentCard component (renders from /api/config/agents)
- AgentGrid component (responsive, real-time status updates)
- tasksSlice Redux reducer (optimistic updates + versioning)
- KanbanBoard component with dnd-kit drag-and-drop
- TaskCard component with visual feedback
- Conflict resolution (version comparison for concurrent updates)
- Keyboard navigation + accessibility (WCAG 2.1 AA)

**Files:** 12 tasks, builds on 04-01 foundation

**Success Criteria:**
- Agent cards render with no hardcoding (all from API)
- Drag task between lanes shows instant feedback
- Task persists after server confirmation
- Concurrent drags auto-resolve via versioning
- Keyboard navigation works (Tab, Arrow, Enter)
- Bundle size increase <150KB

---

### 04-03: Real-Time Collaboration & Live Interactions (Wave 2, ~1 week)

**Goal:** Squad chat, activity feed, task detail modal all synced via WebSocket

**Key Deliverables:**
- SquadChat component (message history, send new messages)
- ActivityFeed component (CoordinationEvent timeline, expandable items)
- TaskDetail modal (full task context, comments, history)
- TaskTimeline component (status change history)
- Message deduplication (no duplicates on reconnect)
- chatSlice and activitiesSlice Redux reducers
- Relative time formatting (date-fns)

**Files:** 11 tasks, leverages 04-01 & 04-02

**Success Criteria:**
- Chat messages send/receive in real-time
- Activity feed shows agent events (<500ms latency)
- Task detail modal shows full context + comments
- No message duplicates on WebSocket reconnect
- Comments persist on page refresh
- Full WCAG 2.1 AA accessibility compliance

---

### 04-04: Configuration APIs & Production Integration (Wave 2, ~1 week)

**Goal:** aofctl serve provides /api/config/* endpoints and static file serving

**Key Deliverables:**
- Axum routes: /api/config/agents, /api/config/tools, /api/config/version
- AGENTS.md and TOOLS.md parsing (YAML → JSON)
- Static file serving for React build (SPA routing fallback)
- File watcher for auto-reload on config change (optional feature)
- Production deployment guide
- Error handling with helpful field path errors (serde_path_to_error)
- Single daemon model (no separate Node.js frontend server)

**Files:** 10 tasks, integrates frontend + backend

**Success Criteria:**
- /api/config/* endpoints return valid JSON
- React build serves from localhost:8080 (no :5173 needed)
- AGENTS.md/TOOLS.md changes reflected in UI
- Single `cargo run` command runs everything
- Production build <2MB total
- Deployment documented and tested

---

## Wave Structure

**Wave 1 (Weeks 1-2):**
- 04-01: Frontend scaffolding and infrastructure
- 04-02: Visualization and user interaction
- Sequential, but 04-02 begins while 04-01 wrap-up (some overlap)

**Wave 2 (Weeks 3-4):**
- 04-03: Real-time collaboration features
- 04-04: Backend APIs and production deployment
- Sequential, but 04-04 can begin while 04-03 testing

## Team & Resources

| Role | Plans | Hours | Notes |
|------|-------|-------|-------|
| Frontend Developer (React/TypeScript) | 04-01, 04-02, 04-03 | 80-100 | Leads component development, hooks |
| Backend Developer (Rust/Axum) | 04-01 (support), 04-04 | 40-50 | Coordinates API contracts, static serving |
| DevOps/Deployment Engineer | 04-04 | 10-20 | Deployment docs, Docker setup (optional) |

**Estimated Total Effort:** 130-170 engineering hours (3-4 weeks with 1-2 developers)

## Critical Dependencies

### From Phase 1 (Already Implemented)
- Axum WebSocket handler at /ws
- CoordinationEvent JSON schema
- tokio::broadcast event channel
- Placeholder /api/config/* endpoints (will be replaced in 04-04)
- aof-memory backend for session persistence

### From Phase 2-3 (Must be Integrated)
- AgentExecutor emitting CoordinationEvent
- FleetCoordinator for multi-agent coordination
- Gateway event normalization (Phase 3)

### New in Phase 4
- React + Vite frontend (new tech stack)
- Redux store (new state management)
- dnd-kit for drag-and-drop (new library)
- Tailwind CSS + shadcn/ui (new component framework)

## Tech Stack

### Backend (Rust)
- Axum 0.7+ (HTTP/WebSocket)
- serde_yaml (config parsing)
- serde_path_to_error (helpful error messages)
- tokio (async runtime)
- tokio::broadcast (event distribution)

### Frontend (JavaScript/TypeScript)
- React 18.x
- TypeScript (strict mode)
- Redux Toolkit + RTK Query
- Vite (build tool)
- dnd-kit (drag-and-drop)
- Tailwind CSS + shadcn/ui
- date-fns (time formatting)
- ws (WebSocket client, via native API)

### Optional/Future
- builder.io (UI generation, integrated post-MVP)
- Leptos WASM (pure Rust frontend, future optimization)

## Success Metrics

### Functional Completeness
- [ ] All 7 requirements (MCUI-01 through MCUI-07) implemented
- [ ] Zero hardcoding of agent/task data (all from APIs)
- [ ] Real-time sync <500ms latency
- [ ] No console errors on typical workflows

### Performance
- [ ] First Contentful Paint <2 seconds
- [ ] Drag-and-drop <100ms perceived latency
- [ ] Bundle size <500KB (gzipped)
- [ ] 60fps scrolling in activity feed

### Quality
- [ ] WCAG 2.1 AA accessibility compliance
- [ ] 80%+ test coverage for core components
- [ ] Zero critical security issues
- [ ] Production deployment documented

### User Experience
- [ ] New user can get running with: `npm install && npm run dev && cargo run -- serve`
- [ ] Configuration changes live-reload (with file watcher)
- [ ] Graceful error messages (field path errors, not generic 500s)
- [ ] Keyboard navigation fully functional

## Known Limitations & Future Work

**Phase 4 Scope (Not Included):**
- User authentication / multi-user support
- Cloud-hosted SaaS deployment
- Mobile-optimized UI (web only, Slack/Discord integrations in Phase 5)
- Advanced analytics / performance profiling
- Leptos WASM optimization (pure Rust frontend)

**Phase 5+ Opportunities:**
- User accounts and workspaces
- Role-based access control (RBAC)
- Agent performance analytics
- Advanced filter/search for tasks and events
- Integration with Slack/Discord for alerts
- AI-generated task suggestions
- Leptos-based pure Rust frontend (for bundle size optimization)

## Handoff Criteria (End of Phase 4)

Before Phase 5 begins:
- [ ] All 4 PLAN.md files executed successfully
- [ ] Phase 4 MVP fully functional (all MCUI requirements met)
- [ ] Deployment guide tested and documented
- [ ] Accessibility audit passed (WCAG 2.1 AA)
- [ ] Performance benchmarks met (latency, bundle size)
- [ ] Code review and merge to main branch
- [ ] Release notes prepared for v0.2.0
- [ ] User documentation updated (docs/mission-control/)

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| WebSocket reconnect issues | Medium | High | useWebSocket hook with exponential backoff, extensive testing |
| Drag-and-drop performance | Low | Medium | Use dnd-kit (battle-tested), avoid custom drag logic |
| Redux state explosion (too many events) | Medium | High | Keep last 500 events, selector memoization |
| Configuration API contract mismatch | Low | Medium | Early integration testing (04-01), API-first design |
| Build size bloat (React + deps) | Low | Medium | Tree-shaking, dynamic imports, dependency audit |
| Accessibility failures | Low | Medium | axe scan + manual testing with screen readers, WCAG checklist |

## References

- **Research:** `/Users/gshah/work/opsflow-sh/aof/.planning/phases/04-mission-control-ui/04-RESEARCH.md`
- **Phase 1:** WebSocket infrastructure, CoordinationEvent schema
- **Phase 2:** Agent execution, memory backends
- **Phase 3:** Gateway, event routing
- **PROJECT.md:** Locked constraints (builder.io, Rust backend focus)

---

**Planning completed:** 2026-02-14
**Ready for execution:** Yes
**Estimated completion:** 2026-03-14 (4 weeks from start)
