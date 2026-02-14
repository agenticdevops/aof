# Builder.io Brief — AOF Milestone 2, Phase 1-4

**Project:** AOF (Agentic Ops Framework) — Humanized Agentic Operations Platform
**Scope:** Complete React web application frontend (4 phases, 4 pages, 40+ components)
**Framework:** React 18 + TypeScript
**Styling:** Tailwind CSS (design tokens provided)
**Deployment:** Vite build system
**Deliverable Timeline:** Phase 1 first (1 week), then Phases 2-4

---

## Overview

Build a beautiful web dashboard where users see their agent teams coordinate in real-time. The system is already production-ready on the backend (Rust daemon with WebSocket streaming, 530+ tests passing). Your job: create the frontend that makes agents feel like team members, not executables.

**Key Philosophy:** Humanize agents. Make their communication, health status, and coordination visible and delightful.

---

## Design System (Locked)

### Colors
- **Primary Green:** `#10b981` (health, success, active)
- **Error Red:** `#ef4444` (problems, unresponsive)
- **Info Blue:** `#3b82f6` (notifications, info)
- **Neutral Gray:** `#6b7280` (text, disabled)
- **Background:** `#ffffff` (light mode), `#1f2937` (dark mode)

### Typography
- **Display (Hero titles):** 32px, weight 700, line-height 1.2
- **Heading:** 20px, weight 600, line-height 1.3
- **Body:** 14px, weight 400, line-height 1.5
- **Mono (API/data):** 12px, family `ui-monospace`, weight 500

### Spacing Scale
- **XS:** 4px | **S:** 8px | **M:** 16px | **L:** 24px | **XL:** 32px | **XXL:** 48px

### Responsive Breakpoints
- **Mobile:** < 640px
- **Tablet:** 640px - 1024px
- **Desktop:** > 1024px

All pages must be fully responsive (mobile-first).

---

## Phase Breakdown

### Phase 1: Onboarding & Configuration UI (1 week, 4 plans)

#### Plan 01-01: Welcome Page + Onboarding Wizard

**Goal:** Users complete setup in <5 minutes without touching YAML.

**Pages to Build:**
1. **Welcome Page** — Hero section with 3 feature cards, CTA "Start Setup"
   - Hero: "Meet Your Agent Squad" tagline, subheading
   - 3 Feature cards: "Real-time Monitoring", "Agent Personas", "Squad Communication"
   - CTA button: "Begin Setup" → routes to wizard Step 1

2. **Onboarding Wizard (4-Step Flow)**
   - Shell component with:
     - Left sidebar: Step progress indicator (1/2/3/4, with checkmarks)
     - Main area: Current step content
     - Navigation buttons: Back/Next/Skip/Complete

   **Step 1: Welcome & Project Setup**
   - Text: "Let's set up your first AOF project"
   - Form fields:
     - Project name (required, min 3 chars)
     - Description (optional, textarea)
     - Button: "Next"
   - Validation: Show inline errors

   **Step 2: Agent Configuration**
   - Text: "Create your first agent"
   - Form fields:
     - Agent name (required)
     - LLM Model (dropdown: Claude, GPT-4, Gemini, etc.)
     - Agent type (radio buttons: Analyst, Coordinator, Specialist)
     - Instructions (textarea, placeholder text provided)
     - Capabilities (multi-select checkboxes)
   - Button: "Next"

   **Step 3: Platform Configuration**
   - Text: "Where should your agent listen for work?"
   - Platform cards (6 platforms, show 2-3 per row):
     - Each platform card: Logo, name, connection status badge
     - Expandable on click to show:
       - Connection input field
       - "Test Connection" button
       - Status indicator (✓/✗)
   - Platforms: Slack, Discord, Telegram, WhatsApp, GitHub, Jira
   - Button: "Next"

   **Step 4: Review & Launch**
   - Show summary of all configuration
   - Cards for: Project, Agent, Platforms (each collapsible)
   - Buttons: "Edit" (goes back), "Launch" (creates config)

**Components to Create:**
- `WelcomePage.tsx` — Hero + feature cards
- `OnboardingWizard.tsx` — 4-step shell
- `WizardProgress.tsx` — Step indicator
- `StepWelcome.tsx` — Step 1
- `StepAgentSetup.tsx` — Step 2
- `StepPlatformConfig.tsx` — Step 3
- `StepReview.tsx` — Step 4
- `FormField.tsx` — Reusable form input with label + error
- `Button.tsx` — 3 variants: primary, secondary, ghost + loading state
- `Input.tsx` — Text input with validation styling
- `TextArea.tsx` — Multi-line input
- `Select.tsx` — Dropdown
- `Radio.tsx` — Radio button group
- `Card.tsx` — Container with 3 elevations (flat, lifted, focused)

**Design Notes:**
- Use step animations when transitioning between steps (fade-in on next, fade-out on back)
- Show success checkmarks when step completed
- Keep wizard width constrained (~500px max)
- Center vertically on desktop, full-width on mobile

---

#### Plan 01-02: Configuration Dashboard (Agents, Tools, Platforms tabs)

**Goal:** Users manage their setup post-onboarding.

**Page: Configuration Dashboard**
- Header: "Configuration" title, last-updated timestamp
- Tab navigation: Agents | Tools | Platforms | Version

**Agents Tab:**
- Search bar (top, full width)
- Grid of agent cards (3 columns on desktop, 1-2 on mobile)
  - Each card shows:
    - Agent name (large, bold)
    - Model (smaller text)
    - Agent type badge (Analyst/Coordinator/Specialist)
    - Capabilities (comma-separated list)
    - Status indicator (✓ healthy)
    - Buttons: "Edit" → modal, "Delete" → confirm dialog
  - Empty state: "No agents yet. Create your first agent."
  - Button (top right): "+ Create Agent"

- **Edit Agent Modal:**
  - Title: "Edit Agent: [name]"
  - Form fields (same as Step 2 of wizard):
    - Agent name
    - Model (dropdown)
    - Type (radio buttons)
    - Instructions (textarea)
    - Capabilities (checkboxes)
  - Buttons: Cancel, Delete, Save

**Tools Tab:**
- Grid of tool cards (2-3 columns)
  - Each tool card:
    - Tool name (bold)
    - Description (1-2 lines)
    - Type badge (API/Integration/Script)
    - Status: "✓ Ready" or "⚠ Needs Config"
    - Button: "Configure" → modal or "View Details"
  - Empty state: "No tools configured"
  - Button (top right): "+ Add Tool"

**Platforms Tab:**
- Platform cards (2 per row on desktop)
  - Each platform card:
    - Platform logo/icon
    - Platform name (Slack, Discord, etc.)
    - Connection status:
      - If connected: "✓ Connected as @username" (green indicator)
      - If not: "Not connected" (gray indicator)
    - Buttons: "Manage" → modal, "Disconnect"
  - Empty state: "No platforms connected"

- **Platform Connection Modal:**
  - Title: "Connect [Platform]"
  - Form varies by platform:
    - **Slack:** Token input (starts with "xoxb-"), "Test Connection" button
    - **Discord:** Bot token, "Test Connection" button
    - **GitHub:** PAT (Personal Access Token) input
    - **Jira:** URL + API token
    - **Telegram:** Bot token + chat ID
    - **WhatsApp:** Business account ID + token
  - Buttons: Cancel, Test Connection, Save
  - After test: Show "✓ Connection successful" or error message

**Components to Create:**
- `ConfigurationPage.tsx` — Main page
- `TabNavigation.tsx` — Tab switcher
- `AgentsTab.tsx` — Agents section
- `AgentCard.tsx` — Agent card in grid
- `AgentDetailModal.tsx` — Edit agent modal
- `ToolsTab.tsx` — Tools section
- `ToolCard.tsx` — Tool card
- `PlatformsTab.tsx` — Platforms section
- `PlatformCard.tsx` — Platform card with status
- `PlatformDetailModal.tsx` — Connection modal (varies by platform)
- `Modal.tsx` — Base modal container
- `Badge.tsx` — Status badges (Analyst, Ready, Connected, etc.)
- `SearchBar.tsx` — Search input
- `EmptyState.tsx` — Empty state illustration + text
- `LoadingSpinner.tsx` — Loading indicator
- `ConfirmDialog.tsx` — Confirmation for destructive actions

**Design Notes:**
- Cards should have hover effects (shadow lift)
- Use badges for status (green for success, yellow for warning)
- Modal should be centered, with overlay
- Tabs should have underline indicator showing active tab
- All forms should validate inline (red text below field)

---

#### Plan 01-03: Form Validation + Error Handling

**Implementation Details Provided by Claude Later**

Goal: All forms validate cleanly with helpful error messages.

---

#### Plan 01-04: Integration Testing + E2E

**Implementation Details Provided by Claude Later**

Goal: End-to-end user flows tested (Welcome → Wizard → Config → Verification).

---

### Phase 2: Mission Control & Squad Chat (2 weeks)

**Pages to Build:**
1. **Mission Control Dashboard**
   - Real-time agent grid showing:
     - Agent name + avatar
     - Current status (Healthy ✓ / Degraded ⚠ / Unresponsive ✗)
     - Response latency (milliseconds)
     - Health pulsing animation
   - Standup feed (expandable responses)
   - Token usage metrics

2. **Squad Chat Panel**
   - Chat interface for agent-to-agent and human-to-agent messaging
   - Message bubbles with persona styling
   - Real-time message arrival
   - User/AI message differentiation

### Phase 3: Fleet Control Dashboard (2 weeks)

**Pages to Build:**
1. **Squad Overview**
   - Agent grouping/squads
   - Team relationships visualization

2. **Task Kanban Board**
   - Columns: Backlog | Assigned | In Progress | Review | Done
   - Drag-and-drop task cards
   - Task detail modal on click

3. **Workflow Builder** (visual DAG)
   - Node-based workflow editor
   - Connect agents in sequence

4. **Performance Analytics**
   - Task completion rates
   - Agent utilization charts
   - Success metrics

### Phase 4: Polish & Integration (1 week)

- Micro-animations on all state changes
- Accessibility audit (WCAG AA)
- Mobile responsiveness verification
- Performance optimization
- Builder.io export

---

## API Integration Points

All API endpoints documented in: `/docs/api/COMPLETE-API-SPECIFICATION.md`

### Phase 1 APIs to Connect

**Configuration Management:**
- `GET /api/config/agents` — List all agents
- `POST /api/config/agents` — Create agent
- `PUT /api/config/agents/{id}` — Update agent
- `DELETE /api/config/agents/{id}` — Delete agent
- `GET /api/config/tools` — List tools
- `GET /api/config/platforms` — List platforms
- `POST /api/config/platforms/{platform}/test` — Test platform connection
- `GET /api/config/version` — Get config version (for checking if persisted)

**Conversation (Agent Creation Flow):**
- `POST /api/conversation/session` — Start setup conversation
- `POST /api/conversation/message` — Send message to setup flow
- `POST /api/conversation/confirm` — Confirm agent creation

### WebSocket Events to Listen

All events come through: `ws://localhost:7777/ws`

**Event Types:**
- `coordination_activity` → HeartbeatResponse, HeartbeatTimeout, StandupResponse, StandupSummary

---

## State Management Pattern (Redux + WebSocket)

### Redux Store Structure

```typescript
// appSlice
{
  navigation: "welcome" | "wizard" | "config" | "missionControl",
  theme: "light" | "dark",
  firstVisit: boolean,
  daemonUrl: "http://localhost:7777"
}

// onboardingSlice (Phase 1)
{
  currentStep: 1 | 2 | 3 | 4,
  project: { name, description },
  agent: { name, model, type, instructions, capabilities },
  platforms: { [platform]: { connected, config } },
  isLoading: boolean,
  error: null | string
}

// configSlice (Phase 1)
{
  agents: Agent[],
  tools: Tool[],
  platforms: Platform[],
  version: string,
  isLoading: boolean,
  error: null | string,
  searchQuery: string
}

// websocketSlice (Phase 2+)
{
  connected: boolean,
  lastError: null | string,
  reconnectAttempt: number
}

// coordinationSlice (Phase 2+)
{
  agents: AgentHealth[],
  latestStandup: StandupResult | null,
  metrics: CoordinationMetrics
}
```

---

## Component Library Requirements

**Priority 1 (Phase 1):**
- Button (primary, secondary, ghost, loading state)
- Input (text, with error styling)
- TextArea (with error styling)
- Select (dropdown)
- Radio (button group)
- Card (3 elevations)
- Modal (centered overlay)
- Badge (status indicators)
- SearchBar (with input icon)
- EmptyState (illustration + message)
- LoadingSpinner (animated)

**Priority 2 (Phase 2+):**
- Avatar (agent persona styling)
- MessageBubble (user/AI styling)
- HeartbeatIndicator (pulsing animation)
- Tabs
- Breadcrumbs
- Tooltip
- Toast/Notification

---

## Handoff Requirements (What You Should Return)

When you complete Phase 1, deliver:

### 1. **Source Code**
- All `.tsx` component files organized in `/web-app/src/components/`
- All `.tsx` page files in `/web-app/src/pages/`
- Redux store setup in `/web-app/src/store/`
- Type definitions in `/web-app/src/types/`

### 2. **Build Artifacts**
- Complete `package.json` with all dependencies
- `vite.config.ts` configured
- `tailwind.config.ts` with design tokens
- `tsconfig.json` strict mode enabled

### 3. **Documentation**
- Storybook stories for all 15+ components (Phase 1 only, showing all variants)
- README explaining component usage patterns
- Migration guide if you had to deviate from spec

### 4. **Component Inventory**
- CSV or JSON listing all components created, their props, and files
- Example usage snippets for each component

### 5. **Design System Export**
- Tailwind config as JSON (colors, spacing, typography)
- Component specs (what props each accepts)
- Dark mode configuration

### 6. **Git Repository**
- Commit your work with clear commit messages
- Push to GitHub branch `phase-1-builder-io` or similar
- Provide PR or branch link

---

## Key Principles for Builder.io

1. **Type Safety:** All components must be fully typed (no `any`)
2. **Accessibility:** Use semantic HTML, ARIA labels where needed
3. **Responsiveness:** Test all components at 320px (mobile), 768px (tablet), 1440px (desktop)
4. **Consistency:** Follow design system exactly (colors, spacing, typography)
5. **Performance:** Use React.memo for expensive components, lazy load where possible
6. **Reusability:** Components must work across all phases (e.g., Button used everywhere)
7. **Error Handling:** All forms should show validation errors inline
8. **Loading States:** Every async operation needs a loading indicator
9. **Dark Mode:** Components should work in both light and dark themes
10. **Builder.io Export:** Component code must be clean and exportable

---

## Timeline & Phases

**Phase 1 (1 week):**
- Welcome page + 4-step wizard
- Config dashboard (Agents/Tools/Platforms tabs)
- Form components + validation
- E2E user flow

**Phase 2 (2 weeks):**
- Mission Control dashboard
- Squad chat interface

**Phase 3 (2 weeks):**
- Fleet Control dashboard
- Kanban board
- Workflow builder
- Performance analytics

**Phase 4 (1 week):**
- Polish & animations
- Accessibility
- Mobile testing
- Export optimization

---

## Handoff After Phase 1

Once Phase 1 is complete and delivered:

1. **Claude takes over for:**
   - API integration (fetch wiring)
   - Redux action/reducer implementation
   - WebSocket client setup
   - Error handling & retry logic
   - Authentication (if needed)
   - Testing (unit, integration, E2E)
   - Performance optimization
   - Deployment setup

2. **You continue for:**
   - Phase 2 (Mission Control)
   - Phase 3 (Fleet Control)
   - Phase 4 (Polish)

3. **Integration Loop:**
   - You provide component code + Storybook
   - Claude wires up state + APIs
   - You receive feedback on styling/UX
   - Iterate until Phase 1 done
   - Move to Phase 2

---

## Resources

- **Complete Frontend Spec:** `/docs/frontend/WEB-APP-SPECIFICATION.md` (2,100 lines)
- **API Specification:** `/docs/api/COMPLETE-API-SPECIFICATION.md` (20+ endpoints)
- **Component Specs:** `/docs/frontend/FRONTEND-COMPONENT-SPEC.md`
- **Coordination API:** `/docs/api/COORDINATION-API-SPEC.md` (WebSocket events)
- **GitHub Repo:** `https://github.com/agenticdevops/aof`

---

## Questions?

Contact: [AOF Project Lead]

Good luck building! Make agents feel human. 🚀
