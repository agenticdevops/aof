# Builder.io Handoff Package

**Everything builder.io needs to build the AOF frontend**

---

## Quick Start for Builder.io

1. **Read this first:** `BUILDER.IO-BRIEF.md` (15 min read)
   - Project overview
   - Design system (colors, fonts, spacing)
   - Pages to build (Phase 1-4)
   - API endpoints to integrate
   - Component library requirements

2. **Detailed specs:** Access these files
   - `/docs/frontend/WEB-APP-SPECIFICATION.md` (Complete UI architecture)
   - `/docs/api/COMPLETE-API-SPECIFICATION.md` (All API endpoints)
   - `/docs/api/COORDINATION-API-SPEC.md` (WebSocket events)

3. **Know what to deliver:** `BUILDER.IO-DELIVERABLES.md` (30 min read)
   - Directory structure
   - File requirements (package.json, tsconfig.json, etc.)
   - Redux store setup
   - Component structure
   - Storybook requirements
   - Quality checklist

4. **Start with Phase 1:**
   - Build Welcome page + 4-step onboarding wizard (Plan 01-01)
   - Build Configuration dashboard with Agents/Tools/Platforms tabs (Plan 01-02)
   - Deliver with full TypeScript, Redux setup, Tailwind CSS

5. **After Phase 1, Claude takes over:**
   - See `CLAUDE-INTEGRATION-HANDOFF.md` (Claude's integration tasks)
   - Claude wires APIs, WebSocket, testing
   - You build Phase 2 on top

---

## File Guide

### For Builder.io

| File | Purpose | Read Time |
|------|---------|-----------|
| **BUILDER.IO-BRIEF.md** | Complete project brief, design system, phase breakdown | 15 min |
| **BUILDER.IO-DELIVERABLES.md** | Exact requirements for Phase 1 handoff | 30 min |
| `/docs/frontend/WEB-APP-SPECIFICATION.md` | Complete UI/UX specification | Reference |
| `/docs/api/COMPLETE-API-SPECIFICATION.md` | API endpoints (read during Phase 1 build) | Reference |

### For Claude (Integration)

| File | Purpose |
|------|---------|
| **CLAUDE-INTEGRATION-HANDOFF.md** | Integration tasks after builder.io delivers Phase 1 |

### Reference Documents

| File | Purpose |
|------|---------|
| `/.planning/ROADMAP.md` | Milestone 2 roadmap (4 phases, 4 weeks) |
| `/.planning/phases/01-onboarding-config-ui/01-01-PLAN.md` | Plan for Welcome + Wizard |
| `/.planning/phases/01-onboarding-config-ui/01-02-PLAN.md` | Plan for Config Dashboard |
| `/.planning/MILESTONE-1-CLOSURE.md` | Context: What was completed in Milestone 1 |

---

## The Overall Process

```
Week 1 (Builder.io)
├─ Phase 1 Frontend Build
│  ├─ Welcome page + 4-step onboarding wizard
│  ├─ Configuration dashboard (Agents/Tools/Platforms tabs)
│  ├─ 30+ reusable components (Button, Input, Modal, etc.)
│  ├─ Redux store setup
│  └─ Deliver: React app, Storybook, documentation
│
└─ Handoff to Claude ✓

Weeks 2-3 (Claude Integration)
├─ API client layer
├─ Redux to API wiring
├─ Form submission logic
├─ Redux Persist (localStorage)
├─ Testing setup (Vitest + MSW)
└─ Phase 1 complete ✓

Week 3 (Builder.io - Phase 2)
├─ Mission Control dashboard
├─ Squad Chat component
└─ Real-time agent health monitoring

Week 4 (Claude Integration Phase 2)
├─ WebSocket integration
├─ Real-time event handling
└─ Phase 2 complete ✓

Weeks 5-6 (Builder.io - Phase 3)
├─ Fleet Control dashboard
├─ Kanban task board
├─ Workflow builder (visual DAG)
└─ Performance analytics

Week 7 (Builder.io - Phase 4)
├─ Micro-animations
├─ Accessibility (WCAG AA)
├─ Mobile testing
└─ Builder.io export
```

---

## Phase 1 Timeline

**Week 1: 5 working days**

| Day | Task | Deliverable |
|-----|------|-------------|
| 1 | Scaffold Vite + React + Redux + Tailwind | `vite.config.ts`, `package.json` ready |
| 2 | Build form components library | Button, Input, Modal, Card, etc. ready |
| 2 | Build Welcome page + Wizard shell | Routes, progress indicator, step components |
| 3 | Build Config Dashboard | Agents/Tools/Platforms tabs with cards |
| 4 | Create Storybook stories | All components documented with variants |
| 5 | Polish + QA + Deliver | PR ready, README complete, tests pass |

---

## Success Criteria for Phase 1

### Functional
- ✅ Welcome page loads and renders correctly
- ✅ 4-step onboarding wizard navigates between steps
- ✅ Config dashboard shows Agents/Tools/Platforms tabs
- ✅ All forms have input validation (visual feedback)
- ✅ Buttons show loading states

### Code Quality
- ✅ Full TypeScript (no `any`)
- ✅ All components have Storybook stories
- ✅ Design tokens from spec match exactly
- ✅ Mobile responsive (3 breakpoints tested)
- ✅ Dark mode toggle works
- ✅ Git history is clean

### Documentation
- ✅ README with setup instructions
- ✅ Component inventory (CSV or JSON)
- ✅ Storybook runs: `pnpm storybook`
- ✅ Dev server runs: `pnpm dev`
- ✅ Build succeeds: `pnpm build`

---

## Key Resources

### Design System (Locked - Don't change)
- **Colors:** Green `#10b981`, Red `#ef4444`, Blue `#3b82f6`, Gray `#6b7280`
- **Typography:** 32px display, 20px heading, 14px body, 12px mono
- **Spacing:** 4px, 8px, 16px, 24px, 32px, 48px
- **All in:** `tailwind.config.ts` (template in DELIVERABLES.md)

### Component Checklist (15+ Priority 1 components)
- Buttons (primary, secondary, ghost, loading)
- Input (text, with validation styling)
- TextArea (with validation)
- Select (dropdown)
- Radio (button group)
- Card (3 elevations)
- Modal (centered overlay)
- Badge (status indicators)
- SearchBar
- EmptyState
- LoadingSpinner
- ConfirmDialog
- FormField (wrapper with label + error)

Plus page components:
- WelcomePage
- OnboardingWizard
- ConfigurationPage

---

## API Integration (Phase 1)

**Builder.io:** Build the UI components
**Claude:** Wire to these endpoints

Base URL: `http://localhost:7777`

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/config/agents` | GET, POST | List/create agents |
| `/api/config/agents/{id}` | PUT, DELETE | Update/delete agent |
| `/api/config/tools` | GET | List tools |
| `/api/config/platforms` | GET | List platforms |
| `/api/config/platforms/{platform}/test` | POST | Test platform connection |
| `/api/conversation/session` | POST | Start setup conversation |
| `/api/conversation/message` | POST | Send message to setup |
| `/api/conversation/confirm` | POST | Confirm agent creation |

Full specs: `/docs/api/COMPLETE-API-SPECIFICATION.md`

---

## After Phase 1: Claude Takes Over

Claude will integrate:

1. **API Client** (`web-app/src/api/`)
   - Typed fetch wrapper
   - Error normalization
   - Request/response interceptors

2. **Redux Integration**
   - Async thunks for all API calls
   - Error handling
   - Loading states

3. **Form Submission**
   - Wire all 6 forms to Redux + API
   - Validation with Zod
   - Success/error messaging

4. **Testing**
   - Vitest setup
   - MSW mock server
   - Integration tests
   - E2E tests

5. **Persistence**
   - Redux Persist to localStorage
   - Recovery on app restart

---

## Questions? Ambiguities?

1. **For design questions:** Reference `/docs/frontend/WEB-APP-SPECIFICATION.md`
2. **For API questions:** Reference `/docs/api/COMPLETE-API-SPECIFICATION.md`
3. **For component structure:** See examples in DELIVERABLES.md
4. **For phase scope:** See `.planning/ROADMAP.md`

**Create an issue or comment on PR if stuck.** Claude will clarify within 24 hours.

---

## Recommended Order

### Read First
1. ✅ This file (README.md)
2. ✅ BUILDER.IO-BRIEF.md
3. ✅ BUILDER.IO-DELIVERABLES.md

### Keep Handy During Build
- `/docs/frontend/WEB-APP-SPECIFICATION.md` (Reference)
- `/docs/api/COMPLETE-API-SPECIFICATION.md` (Reference)

### After Phase 1
- CLAUDE-INTEGRATION-HANDOFF.md (Claude's integration tasks)

---

## Project Context

### Milestone 1 ✅ Complete
- 8 phases, 35 plans delivered
- 530+ tests passing
- API ready (`aofctl serve` runs on localhost:7777)
- Rust backend production-ready

### Milestone 2 🚀 Starting
- Phase 1: Onboarding & Config UI (1 week) ← YOU ARE HERE
- Phase 2: Mission Control & Squad Chat (2 weeks)
- Phase 3: Fleet Control Dashboard (2 weeks)
- Phase 4: Polish & Integration (1 week)

### Your Role (Builder.io)
Build beautiful, fully-typed React components that make agents feel human. Claude handles integration, state management, and testing.

---

## Let's Go! 🚀

**Phase 1 is a 1-week sprint.**

Start with BUILDER.IO-BRIEF.md, then dive into Phase 1 plans.

The backend is ready. The specs are complete. Build something amazing.

---

**Questions? Issues? PR feedback?**

Claude is here to help. Create an issue in the GitHub repo or comment on your PR.

**Good luck! Make agents feel human.** ✨
