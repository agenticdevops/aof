# Phase 2: Mission Control & Squad Chat — Planning Complete

**Status:** ✅ All 5 plans created and ready for execution
**Phase Duration:** 2 weeks (estimated)
**Wave Structure:** 5 sequential waves (each plan is a wave)

---

## Phase Overview

Phase 2 transforms AOF into a **humanized agentic ops platform** where agents feel like visible team members with real-time coordination and visible communication.

**Phase 1.5 Delivered:**
- 3-step onboarding complete
- Xops auto-created and ready
- Specialist bot templates available
- Tool auto-discovery working
- Approval workflow in place

**Phase 2 Delivers:**
- Mission Control dashboard (real-time agent monitoring)
- Squad Chat interface (agent-to-agent and human-to-agent messaging)
- WebSocket real-time event handling
- Persona system (agent visual identities)
- Complete end-to-end integration

---

## Plans Overview

### Wave 1: Plan 02-01 — Mission Control Dashboard Redesign
**Objective:** Create the main Mission Control page showing all agents in grid layout with real-time status, health indicators, and metrics.

**Key Deliverables:**
- Mission Control page component (responsive grid layout)
- AgentCard component with metrics and status indicators
- AgentDetailModal for expanded agent view
- Redux dashboard slice for state management
- CSS animations for heartbeat and status transitions
- 8 implementation tasks, 15+ tests

**Files Created:** 7
**Tests:** 15+
**Timeline:** 3-4 days

---

### Wave 2: Plan 02-02 — Squad Chat Component & Messaging
**Objective:** Build the Squad Chat interface where humans and agents communicate in real-time with persona-based message styling.

**Key Deliverables:**
- SquadChat page with 2-column layout (feed + sidebar)
- MessageFeed component with auto-scroll and virtualization
- MessageCard with persona-based styling
- MessageInput with send validation
- SquadMemberList sidebar component
- Redux chat slice for message management
- 8 implementation tasks, 12+ tests

**Files Created:** 7
**Tests:** 12+
**Timeline:** 3-4 days

---

### Wave 3: Plan 02-03 — Real-Time Event Handling & Animations
**Objective:** Implement WebSocket event handling for real-time updates with smooth animations for status changes, messages, typing indicators.

**Key Deliverables:**
- WebSocket event type definitions
- Toast notification system
- useWebSocket custom hook with auto-reconnect
- Redux middleware for event handling
- Updated Redux slices with event handlers
- Connection status indicator in header
- Comprehensive integration tests
- 8 implementation tasks, 18+ tests

**Files Created:** 6
**Tests:** 18+
**Timeline:** 3-4 days

---

### Wave 4: Plan 02-04 — Persona-Based Styling & Customization
**Objective:** Implement comprehensive persona system so agents visually express their personality through colors, fonts, and icons.

**Key Deliverables:**
- Persona type definitions (analyst, coordinator, specialist, responder)
- Color scheme data (light + dark mode variants)
- Persona styling utility functions
- AgentAvatar component with persona styling
- Updated AgentCard and MessageCard for persona colors
- Redux persona slice
- 8 implementation tasks, 15+ tests

**Files Created:** 7
**Tests:** 15+
**Timeline:** 2-3 days

---

### Wave 5: Plan 02-05 — WebSocket Integration & Real-Time Sync
**Objective:** Final integration of all real-time features. Ensure Mission Control and Squad Chat stay in perfect sync with backend agent state via WebSocket.

**Key Deliverables:**
- WebSocket client with event parsing
- Complete middleware with all event handlers
- Message delivery status tracking (sent, received, read)
- Offline message queueing and retry
- Complete Redux slice updates
- Comprehensive E2E tests covering entire flow
- Performance monitoring and optimization
- Production readiness validation
- 8 implementation tasks, 10+ tests

**Files Created:** 5
**Tests:** 10+
**Timeline:** 4-5 days

---

## Implementation Approach

### Task Structure (Across All 5 Plans)
- **40 total implementation tasks** (8 per plan)
- **60+ comprehensive tests** (15+/12+/18+/15+/10+ per plan)
- **25+ source files created/modified**
- **No `any` types** — full TypeScript strict mode
- **80%+ test coverage** on critical components

### Quality Standards
- ✅ All tasks have specific, measurable acceptance criteria
- ✅ Every component typed with TypeScript interfaces
- ✅ Every Redux action/reducer fully typed
- ✅ Every test verifies behavior, not just compilation
- ✅ Performance targets specified (<100ms latency)
- ✅ Responsive design validated (320px, 768px, 1440px)
- ✅ Dark mode support throughout
- ✅ Accessibility considered (ARIA labels, keyboard nav, contrast)

### Dependencies & Sequencing

```
Wave 1: Plan 02-01 (Dashboard) — no dependencies
   ↓
Wave 2: Plan 02-02 (Chat) — depends on 02-01
   ↓
Wave 3: Plan 02-03 (WebSocket) — depends on 02-01, 02-02
   ↓
Wave 4: Plan 02-04 (Personas) — depends on 02-01, 02-02, 02-03
   ↓
Wave 5: Plan 02-05 (Integration) — depends on 02-01, 02-02, 02-03, 02-04
```

Each plan can execute after its dependencies complete. Within a wave, all tasks can run in parallel.

---

## Success Metrics

### Performance
- WebSocket event → Redux dispatch: **< 100ms**
- Redux dispatch → React render: **< 50ms**
- Total event → UI update: **< 150ms**
- Message delivery: **< 150ms** from send to display
- Animations: **60fps**, smooth (no jank)

### Functionality
- ✅ Mission Control displays agent grid with real-time metrics
- ✅ Squad Chat shows messages in real-time
- ✅ Standup summaries appear in both dashboards automatically
- ✅ Offline messages queue and retry on reconnect
- ✅ Message delivery status tracked (sent, received, read)
- ✅ Connection status indicator in header
- ✅ Dark mode toggle works on all pages
- ✅ Responsive at 320px, 768px, 1440px

### Testing
- **Total tests:** 60+ across all plans
- **Test coverage:** 80%+ on critical components
- **E2E coverage:** Complete user flow from agent event to UI update
- **Error scenarios:** All handled gracefully
- **Performance tests:** Latency validated for all critical paths

### Code Quality
- **TypeScript:** Strict mode, zero errors
- **No `any` types:** Full type safety
- **Patterns:** Follow Phase 1 conventions (Redux, Tailwind, components)
- **Documentation:** Clear task descriptions and verification criteria
- **Maintainability:** Modular components, reusable utilities, clean separation of concerns

---

## Architecture Overview

### Component Hierarchy
```
App
├── Layout (header + navigation)
│   └── Connection status indicator
├── Routes
│   ├── WelcomePage
│   ├── OnboardingWizard
│   ├── ConfigurationPage
│   ├── MissionControl (NEW - Plan 01)
│   │   ├── AgentGrid (plan 01)
│   │   │   └── AgentCard (plan 01)
│   │   │       └── AgentAvatar (plan 04)
│   │   └── AgentDetailModal (plan 01)
│   └── SquadChat (NEW - Plan 02)
│       ├── MessageFeed (plan 02)
│       │   └── MessageCard (plan 02)
│       │       └── AgentAvatar (plan 04)
│       ├── MessageInput (plan 02)
│       └── SquadMemberList (plan 02)
└── WebSocket Middleware (plan 03)
```

### Redux State
```
{
  app: { navigation, theme, firstVisit, connectionStatus },
  onboarding: { currentStep, project, agent, platforms },
  config: { agents, tools, platforms },
  dashboard: { agents, selectedAgent, isLoading }, // NEW - plan 01
  chat: { messages, squadMembers, searchQuery, typingAgent }, // NEW - plan 02
  personas: { personas, defaultPersonas } // NEW - plan 04
}
```

### WebSocket Events (Plan 03+)
```
HeartbeatEvent → updateAgentMetrics
StandupEvent → addMessage
MessageEvent → addMessage
AgentStatusChangeEvent → updateAgentStatus
AgentJoinEvent → addSquadMember
AgentLeaveEvent → removeSquadMember
TypingIndicatorEvent → setTypingAgent
AnnouncementEvent → addMessage + broadcast
```

### Persona System (Plan 04+)
```
PersonaType: 'analyst' | 'coordinator' | 'specialist' | 'responder'

Colors (light + dark):
- Analyst: #3b82f6 (blue)
- Coordinator: #10b981 (emerald)
- Specialist: #8b5cf6 (violet)
- Responder: #ef4444 (red)

Typography:
- Coordinator: bold
- Others: regular
```

---

## Key Features

### Mission Control Dashboard
- Real-time agent grid (4-column desktop, 2-column tablet, 1-column mobile)
- Agent cards with name, role, status, avatar, metrics
- Pulsing heartbeat animation on active agents
- Status color coding (green/yellow/red)
- Click → expanded detail modal
- Empty state when no agents
- Dark mode support

### Squad Chat
- 2-column layout (message feed + squad member sidebar)
- Messages in chronological order with relative timestamps
- Auto-scroll to bottom on new messages
- Message search (filter by text)
- Persona-based styling (different colors/fonts per agent type)
- Message input with send button and validation
- Squad member list with online/offline indicators
- Agent/Human badges
- Typing indicator with animated dots

### Real-Time Synchronization
- WebSocket connection established on app startup
- Auto-reconnection with exponential backoff (3s, 6s, 12s, 30s)
- HeartbeatEvent updates agent metrics within 100ms
- StandupEvent adds messages and updates status
- MessageEvent delivers messages in real-time
- Smooth status transition animations
- Toast notifications for key events (new message, agent join)
- Typing indicators while agent responds
- Connection status indicator in header (green/yellow/gray/red)

### Offline Support
- Messages queue when offline
- Auto-retry on reconnection
- Message delivery status tracked (pending → sent → received → read)
- Visual feedback (checkmarks showing status)
- No data loss (queued messages persist until sent)

### Persona System
- Each agent type (analyst, coordinator, specialist, responder) has unique colors
- Avatar components render with persona background color
- Message cards styled with agent persona colors
- Typography varies (coordinator agents bold)
- Dark mode variants for all colors
- Optional customization UI (color picker for personas)

---

## File Summary

### New Pages (2)
- `src/pages/MissionControl.tsx` — Main dashboard
- `src/pages/SquadChat.tsx` — Chat interface

### New Components (11)
- `src/components/dashboard/AgentGrid.tsx` — Responsive agent grid
- `src/components/dashboard/AgentCard.tsx` — Agent card with metrics
- `src/components/dashboard/AgentDetailModal.tsx` — Agent detail view
- `src/components/chat/MessageFeed.tsx` — Message list with auto-scroll
- `src/components/chat/MessageCard.tsx` — Individual message
- `src/components/chat/MessageInput.tsx` — Message input with send
- `src/components/chat/SquadMemberList.tsx` — Squad member sidebar
- `src/components/common/AgentAvatar.tsx` — Persona-styled avatar
- `src/components/common/Toast.tsx` — Toast notification
- Plus 2 more for middleware/hooks

### New Redux Slices (3)
- `src/store/slices/dashboardSlice.ts` — Dashboard state
- `src/store/slices/chatSlice.ts` — Chat state
- `src/store/slices/personaSlice.ts` — Persona management

### New Types & Utils (7)
- `src/types/dashboard.ts` — Dashboard types
- `src/types/chat.ts` — Chat types
- `src/types/personas.ts` — Persona types
- `src/types/events.ts` — WebSocket event types
- `src/data/colorSchemes.ts` — Color palettes
- `src/utils/personaStyles.ts` — Styling utilities
- `src/styles/animations.css` — CSS animations

### New Middleware & Hooks (3)
- `src/middleware/websocketMiddleware.ts` — Event handling
- `src/hooks/useWebSocket.ts` — WebSocket connection
- `src/api/websocket.ts` — WebSocket client

### Updated Files (3)
- `src/App.tsx` — Add routes and WebSocket init
- `src/components/layout/Layout.tsx` — Navigation links
- `src/store/store.ts` — Add new slices

### Tests (60+)
- `src/test/unit/dashboard.test.tsx` — Dashboard component tests
- `src/test/unit/chat.test.tsx` — Chat component tests
- `src/test/unit/personas.test.ts` — Persona system tests
- `src/test/integration/websocket.test.tsx` — WebSocket integration
- `src/test/integration/events.test.ts` — Event handling
- `src/test/integration/offline.test.tsx` — Offline scenarios
- `src/test/e2e/mission-control.test.tsx` — Dashboard flow
- `src/test/e2e/squad-chat.test.tsx` — Chat flow
- `src/test/e2e/complete-flow.test.tsx` — End-to-end integration
- `src/test/e2e/production.test.tsx` — Production readiness

---

## Execution Timeline

| Week | Focus | Plans | Tasks | Tests | Output |
|------|-------|-------|-------|-------|--------|
| 1 | Dashboards & Chat UI | 01-02 | 16 | 27 | Pages, components, Redux |
| 1.5 | Real-time & Personas | 03-04 | 16 | 33 | WebSocket, persona system |
| 2 | Integration & Testing | 05 | 8 | 10+ | E2E tests, validation |

---

## Known Limitations & Future Work

### Not in Phase 2
- Voice/audio in chat (Phase 3+)
- Chat message threading (Phase 3)
- Advanced analytics/charts (Phase 3)
- Fleet control features (Phase 3)
- Task management/kanban (Phase 3)
- Slack/Discord/Telegram integration (Phase 3+)

### Possible Enhancements (Post-Launch)
- Message reactions (emoji reactions)
- Pinned messages
- Channel topics/descriptions
- Agent availability/do-not-disturb status
- Read-only mode for observers
- Broadcast announcements with ACK tracking

### Performance Optimization (Future)
- Message database pagination (currently in-memory)
- Metrics history/trending (sparklines)
- Agent memory usage monitoring
- WebSocket message compression (large events)

---

## Next Steps

1. **Execute Plan 02-01** (Wave 1)
   - Create Mission Control dashboard and components
   - Build Redux dashboard slice
   - Write 15+ tests
   - Timeline: 3-4 days

2. **Execute Plan 02-02** (Wave 2)
   - Create Squad Chat page and messaging components
   - Build Redux chat slice
   - Write 12+ tests
   - Timeline: 3-4 days

3. **Execute Plan 02-03** (Wave 3)
   - Implement WebSocket event handling
   - Create Redux middleware and hooks
   - Write 18+ integration tests
   - Timeline: 3-4 days

4. **Execute Plan 02-04** (Wave 4)
   - Build persona system
   - Update components with persona styling
   - Write 15+ tests
   - Timeline: 2-3 days

5. **Execute Plan 02-05** (Wave 5)
   - Complete WebSocket integration
   - Write comprehensive E2E tests
   - Validate performance and production readiness
   - Timeline: 4-5 days

**Total Phase 2 Duration:** 2-2.5 weeks
**Total Implementation Tasks:** 40
**Total Tests:** 60+

---

## Resources

All plans are in `.planning/phases/02-mission-control-squad-chat/`:
- `02-01-PLAN.md` — Mission Control Dashboard
- `02-02-PLAN.md` — Squad Chat Component
- `02-03-PLAN.md` — Real-Time Event Handling
- `02-04-PLAN.md` — Persona System
- `02-05-PLAN.md` — WebSocket Integration

---

**Phase 2 Planning Complete ✅**

Ready for execution by Claude Code agents.
