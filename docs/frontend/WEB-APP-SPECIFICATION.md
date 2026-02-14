# AOF Web Application Specification

**Version:** 1.0
**Status:** Ready for builder.io Implementation
**Last Updated:** 2026-02-14
**Scope:** Complete AOF Mission Control Web UI

---

## Overview

This document defines the complete web application architecture for AOF Mission Control. It covers:

- **Onboarding Wizard** - First-time setup and agent creation
- **Configuration Dashboard** - Agent, tool, and platform management
- **Mission Control** - Real-time monitoring and coordination
- **Fleet Control** - Multi-agent orchestration (Phase 2)

---

## Table of Contents

1. [Architecture](#architecture)
2. [Data Flow](#data-flow)
3. [Page Structure](#page-structure)
4. [Onboarding Wizard](#onboarding-wizard)
5. [Configuration Dashboard](#configuration-dashboard)
6. [Mission Control Dashboard](#mission-control-dashboard)
7. [Design System](#design-system)
8. [Component Library](#component-library)
9. [State Management](#state-management)
10. [Integration Checklist](#integration-checklist)

---

## Architecture

### Application Structure

```
web-app/
├── src/
│   ├── pages/
│   │   ├── WelcomePage.tsx          # First visit or after reset
│   │   ├── OnboardingWizard.tsx     # Agent/platform setup flow
│   │   ├── DashboardLayout.tsx      # Main app shell
│   │   ├── ConfigurationPage.tsx    # Agent/tool/platform config
│   │   ├── MissionControlPage.tsx   # Real-time monitoring
│   │   └── FleetControlPage.tsx     # Multi-agent orchestration (v1.1)
│   ├── components/
│   │   ├── onboarding/              # Wizard components
│   │   ├── config/                  # Configuration components
│   │   ├── mission-control/         # Mission control components
│   │   ├── fleet/                   # Fleet control components
│   │   ├── common/                  # Reusable UI components
│   │   └── layout/                  # Navigation, sidebar, headers
│   ├── hooks/
│   │   ├── useAPI.ts                # REST API client
│   │   ├── useWebSocket.ts          # WebSocket connection
│   │   ├── useAuth.ts               # Authentication state
│   │   └── useCoordination.ts       # Coordination data fetching
│   ├── store/
│   │   ├── appSlice.ts              # App state (theme, navigation)
│   │   ├── configSlice.ts           # Agent/tool/platform config
│   │   ├── coordinationSlice.ts     # Heartbeat, standup, metrics
│   │   ├── conversationSlice.ts     # Agent creation conversations
│   │   └── store.ts                 # Redux configuration
│   ├── types/
│   │   ├── api.ts                   # API response types
│   │   ├── domain.ts                # Domain types (Agent, Tool, etc)
│   │   └── ui.ts                    # UI state types
│   ├── services/
│   │   ├── apiClient.ts             # Axios/Fetch wrapper
│   │   ├── websocketClient.ts       # WebSocket manager
│   │   └── eventBus.ts              # Client-side event handling
│   └── App.tsx                      # Main entry point
├── public/
│   ├── index.html
│   └── favicon.ico
└── package.json
```

### Technology Stack

- **Framework:** React 18+ with TypeScript
- **State Management:** Redux Toolkit + Redux Persist
- **Real-time:** WebSocket (native, no Socket.io)
- **HTTP:** Axios or Fetch API
- **Styling:** Tailwind CSS + Custom Design System
- **Components:** Shadcn/ui or Headless UI
- **Forms:** React Hook Form + Zod validation
- **Routing:** React Router v6
- **Build:** Vite + TypeScript

---

## Data Flow

### Initialization Flow

```
App Start
  ↓
Load Redux State (from localStorage)
  ↓
[Config exists?]
  ├─ YES: Load ConfigurationPage
  └─ NO: Show OnboardingWizard
       ↓
     Complete Wizard
       ↓
     Fetch /api/config/agents
       ↓
     Store to Redux
       ↓
     Navigate to DashboardLayout
       ↓
     Connect WebSocket (/ws)
       ↓
     Fetch /api/coordination/* on demand
```

### Real-time Update Flow

```
WebSocket Message Received
  ↓
Parse coordination_activity event
  ↓
[Event Type]
  ├─ HeartbeatResponse: Update Redux agent health
  ├─ StandupResponse: Append to standup feed
  ├─ StandupSummary: Mark standup complete
  └─ ExecutionEvent: Log execution
  ↓
Redux subscribers trigger component re-render
```

---

## Page Structure

### Welcome Page

**Route:** `/` (when no config exists)

**Purpose:** First impression and next-step guidance

**Components:**
- Logo and brand message
- Feature overview (3-4 key benefits)
- "Get Started" button → OnboardingWizard
- Optional: Demo video or quick tour

**API Calls:**
- `GET /api/config/version` (check if config exists)

---

### Onboarding Wizard

**Route:** `/onboarding`

**Purpose:** Guided setup for first-time users

**Structure:** Multi-step form wizard

#### Step 1: Welcome
- User name (optional)
- Preferred workspace name
- Next action choice:
  - Create new agent
  - Use existing agents
  - Configure platform integrations

#### Step 2: Agent Setup (if "Create new agent")
- Launch conversational agent creation
- `POST /api/conversation/session`
- Multi-turn conversation
- Save created agent

#### Step 3: Platform Configuration (if "Configure platforms")
- Select platforms (Slack, Discord, etc)
- Input authentication credentials
- Test connection
- Optional: Create first workflow

#### Step 4: Review & Launch
- Summary of setup
- Option to create additional items
- "Launch" button → DashboardLayout

**API Calls:**
- `POST /api/conversation/session` (agent creation)
- `POST /api/conversation/message` (multi-turn)
- `POST /api/conversation/confirm` (save agent)
- `POST /api/config/*` (save platform config - future)

---

### Configuration Dashboard

**Route:** `/config`

**Purpose:** Manage agents, tools, and platform integrations

**Sections:**

#### Agents Tab
- List all agents (cards)
  - Agent name, description, model
  - Actions: View, Edit, Delete, Test
- "+ Create Agent" button → conversational flow
- Search and filter
- Agent detail modal:
  - Edit name/description
  - Adjust model/parameters
  - View capabilities
  - Test with quick prompt

#### Tools Tab
- List all available tools
  - Tool name, type (local/mcp), provider
  - Status (connected/disconnected)
- Add tool dialog
- Tool configuration options

#### Platforms Tab
- List connected platforms
  - Slack, Discord, Telegram, WhatsApp, GitHub, Jira
- "+ Add Platform" button
- Platform detail modal:
  - Show connection status
  - Edit credentials
  - Test connection
  - Remove platform

**API Calls:**
- `GET /api/config/agents`
- `GET /api/config/tools`
- `GET /api/config/version`
- `GET /api/agents/:id/metrics`

---

### Mission Control Dashboard

**Route:** `/mission-control`

**Purpose:** Real-time monitoring of agent health and coordination

**Layout:**
```
┌─────────────────────────────────────────────┐
│ Header: "Mission Control" | Mode: Full      │
├─────────────────────────────────────────────┤
│                                             │
│ ┌──────────────────┐ ┌──────────────────┐   │
│ │ Coordination     │ │ Agent Health     │   │
│ │ Status Card      │ │ Summary          │   │
│ │ - Mode badge     │ │ - Total agents   │   │
│ │ - Overhead gauge │ │ - Healthy count  │   │
│ │ - Token meter    │ │ - Issues count   │   │
│ └──────────────────┘ └──────────────────┘   │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Heartbeat Dashboard (Agent Grid)        │ │
│ │                                         │ │
│ │  [Agent Card] [Agent Card] [Agent Card] │ │
│ │  Status | Latency | Last HB            │ │
│ │  [Agent Card] [Agent Card] [Agent Card] │ │
│ └─────────────────────────────────────────┘ │
│                                             │
│ ┌─────────────────────────────────────────┐ │
│ │ Standup Feed (Live Responses)           │ │
│ │                                         │ │
│ │ [Agent A Response Card]                 │ │
│ │ [Agent B Response Card]                 │ │
│ │ [Standup Summary Card]                  │ │
│ │                                         │ │
│ │ [Trigger Standup Button]                │ │
│ └─────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

**Components:**

#### Coordination Status Card
- Mode dropdown (Full/Standard/Reduced/HeartbeatOnly/Disabled)
- Token overhead gauge (color-coded)
  - Green: <20%
  - Yellow: 20-30%
  - Red: >30%
- Token breakdown (heartbeat, standup, production)
- Auto-degrade toggle

#### Heartbeat Dashboard
- Agent health grid (responsive: 1-3 columns)
- Agent cards show:
  - Status badge (green/yellow/red)
  - Agent name
  - Last heartbeat timestamp
  - Response time (ms)
  - Pulsing animation for unresponsive
- Refresh button
- Filters (all, healthy, degraded, unresponsive)

#### Standup Feed
- Chronological list of standup responses
- Agent response cards (expandable):
  - Agent name
  - "What I did" (summary)
  - "What I'm doing" (current task)
  - "Blockers" (list of blockers)
  - Timestamp
  - Token count
- Standup summary card at top
- "Trigger Standup" button + last triggered time
- Rate limit warning (1 per 10 seconds)

**API Calls:**
- `GET /api/coordination/health` (load on mount)
- `GET /api/coordination/metrics` (polling every 30s)
- `GET /api/coordination/standup/latest` (load on mount)
- `POST /api/coordination/standup/trigger` (user triggered)
- `POST /api/coordination/mode` (mode change)
- `WebSocket /ws` (real-time events)

**Polling Strategy:**
```typescript
// On component mount
useEffect(() => {
  // Load initial state
  dispatch(fetchHealth());
  dispatch(fetchLatestStandup());

  // Poll metrics every 30 seconds
  const metricsInterval = setInterval(() => {
    dispatch(fetchMetrics());
  }, 30000);

  return () => clearInterval(metricsInterval);
}, [dispatch]);

// Real-time WebSocket
useEffect(() => {
  connectWebSocket();
  return () => disconnectWebSocket();
}, []);
```

---

### Fleet Control Dashboard (Phase 2)

**Route:** `/fleet-control`

**Purpose:** Multi-agent orchestration and workflow management

**Planned Components:**
- Workflow builder (visual DAG)
- Agent grouping (squads)
- Task assignment
- Execution monitoring
- Performance analytics

*Detailed spec: Deferred to Phase 8+*

---

## Onboarding Wizard

### Step-by-Step Wizard Design

#### Step 1: Welcome
```
┌──────────────────────────────────┐
│ 🚀 Welcome to AOF Mission Control│
│                                  │
│ Let's get you set up in 2 minutes│
│                                  │
│ [Account Name]                   │
│ [Workspace Name]                 │
│                                  │
│ What would you like to do?       │
│ ○ Create a new agent             │
│ ○ Configure a platform           │
│ ○ Review existing setup          │
│                                  │
│ [< Back] [Next >]                │
└──────────────────────────────────┘
```

#### Step 2a: Conversational Agent Creation
```
┌──────────────────────────────────────────┐
│ Let's create your first agent            │
│                                          │
│ Describe what you want this agent to do: │
│                                          │
│ [Text input...]                          │
│                                          │
│ "I need a Kubernetes expert that can..." │
│                                          │
│ [Next >]                                 │
└──────────────────────────────────────────┘
```

Multi-turn conversation flow:
1. User describes desired agent
2. System asks clarifying questions (4 turns)
3. Review generated agent specs
4. Confirm and save

#### Step 3: Platform Configuration
```
┌──────────────────────────────────┐
│ Connect Messaging Platforms      │
│                                  │
│ ☑ Slack                          │
│   [Bot Token Input]              │
│   [Signing Secret Input]         │
│   [Test Connection Button]       │
│                                  │
│ ☐ Discord                        │
│ ☐ Telegram                       │
│ ☐ GitHub                         │
│                                  │
│ [< Back] [Next >]                │
└──────────────────────────────────┘
```

#### Step 4: Review
```
┌──────────────────────────────────┐
│ You're all set! 🎉               │
│                                  │
│ Summary:                         │
│ • 1 Agent created (k8s-expert)  │
│ • 1 Platform connected (Slack)  │
│ • Ready to monitor 24/7         │
│                                  │
│ [Edit] [Create Another] [Launch]│
└──────────────────────────────────┘
```

---

## Configuration Dashboard

### Component Hierarchy

```
ConfigurationPage
├── TabNavigation
│   ├── Agents Tab
│   ├── Tools Tab
│   └── Platforms Tab
├── AgentsSection
│   ├── SearchBar
│   ├── FilterButtons
│   ├── AgentGrid
│   │   ├── AgentCard (x N)
│   │   │   ├── Name, Description
│   │   │   ├── Model
│   │   │   ├── Actions (View, Edit, Delete, Test)
│   │   │   └── Metrics (success rate, tokens)
│   │   └── CreateAgentButton
│   └── AgentDetailModal
│       ├── Agent Info
│       ├── Model Selection
│       ├── Parameters
│       └── Test Prompt Input
├── ToolsSection
│   ├── ToolsList
│   ├── ToolCard (x N)
│   └── AddToolButton
└── PlatformsSection
    ├── PlatformsList
    ├── PlatformCard (x N)
    └── AddPlatformButton
```

### Agent Card Component

```typescript
interface AgentCardProps {
  agent: AgentConfig;
  metrics?: AgentMetrics;
  onView: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onTest: () => void;
}

// Visual structure:
// ┌─────────────────────────┐
// │ [Agent Icon] Agent Name │
// │ Description line        │
// │                         │
// │ Model: google:gemini    │
// │ Status: ✓ Healthy       │
// │ Success: 95% Avg: 2.3s  │
// │                         │
// │ [View][Edit][Test][...] │
// └─────────────────────────┘
```

---

## Mission Control Dashboard

### Component Hierarchy

```
MissionControlPage
├── Header
│   ├── Title: "Mission Control"
│   ├── Mode Selector
│   └── Last Updated Timestamp
├── StatusRow
│   ├── CoordinationStatusCard
│   │   ├── ModeDropdown
│   │   ├── OverheadGauge
│   │   ├── TokenBreakdown
│   │   └── AutoDegradeToggle
│   └── AgentHealthSummary
│       ├── Total Count
│       ├── Healthy Count
│       ├── Degraded Count
│       └── Unresponsive Count
├── HeartbeatDashboard
│   ├── FilterButtons (All, Healthy, Degraded, Unresponsive)
│   ├── RefreshButton
│   ├── AgentGrid (responsive)
│   │   └── AgentHealthCard (x N)
│   │       ├── Status Badge (animated)
│   │       ├── Agent Name
│   │       ├── Last Heartbeat
│   │       └── Response Time
│   └── EmptyState (if no agents)
├── StandupSection
│   ├── Header
│   │   ├── "Recent Standup"
│   │   ├── Last Triggered Time
│   │   ├── Trigger Button
│   │   └── Rate Limit Warning
│   ├── SummaryCard (if available)
│   │   ├── Summary Text
│   │   ├── Responses Collected Count
│   │   └── Timestamp
│   └── ResponseFeed
│       ├── ResponseCard (x N)
│       │   ├── Agent Name
│       │   ├── What I Did (expandable)
│       │   ├── What I'm Doing
│       │   ├── Blockers (pill badges)
│       │   ├── Timestamp
│       │   └── Token Count
│       └── EmptyState
└── ReconnectingIndicator (if WS disconnected)
```

### CoordinationStatusCard

```
┌─────────────────────────────────┐
│ Coordination Status             │
│                                 │
│ Mode: [Full ▼]                  │
│                                 │
│ Overhead: 4.2%                  │
│ [████░░░░░░░░░░░░░░░░░░]        │
│ (green, <20%)                   │
│                                 │
│ Token Breakdown:                │
│ • Coordination: 4,200 (4.2%)    │
│ • Production: 95,800 (95.8%)    │
│ • Heartbeat: 2,100              │
│ • Standup: 2,100                │
│                                 │
│ ☑ Auto-degrade enabled          │
│ Max overhead: 30%               │
│                                 │
│ ⟳ Update: 30 seconds ago        │
└─────────────────────────────────┘
```

### AgentHealthCard

```
┌──────────────────┐
│ ✓ (green)        │
│ kubo             │
│ Kubernetes       │
│                  │
│ HB: 09:30:15     │
│ Response: 145ms  │
│                  │
│ Healthy          │
└──────────────────┘

OR (Unresponsive):

┌──────────────────┐
│ ✗ (red, pulsing) │
│ network-watch    │
│ Network Watcher  │
│                  │
│ HB: 09:28:00     │
│ Misses: 3        │
│                  │
│ Unresponsive     │
└──────────────────┘
```

### StandupResponseCard

```
┌────────────────────────────────────┐
│ k8s-monitor | 09:00:15             │
│                                    │
│ What I did:                        │
│ > Monitored 12 pods across 3ns...  │
│ [Show More]                        │
│                                    │
│ What I'm doing:                    │
│ > Diagnosing pod logs              │
│                                    │
│ Blockers: None                     │
│                                    │
│ Tokens: 284 | Expand ▼             │
└────────────────────────────────────┘
```

---

## Design System

### Colors

**Primary Palette:**
- Brand Green: `#10b981`
- Brand Blue: `#3b82f6`
- Neutral Gray: `#6b7280`

**Status Colors:**
- Healthy: `#10b981` (green)
- Degraded: `#f59e0b` (amber)
- Unresponsive: `#ef4444` (red)
- Neutral: `#9ca3af` (gray)

**Semantic:**
- Success: `#10b981`
- Warning: `#f59e0b`
- Error: `#ef4444`
- Info: `#3b82f6`

### Typography

- **Display**: 32px, weight 700 (headings)
- **Heading**: 24px, weight 600 (section titles)
- **Subheading**: 18px, weight 600 (subsection titles)
- **Body**: 14px, weight 400 (standard text)
- **Caption**: 12px, weight 500 (secondary text)
- **Mono**: 12px, weight 400 (code, timestamps)

### Spacing

- **XS**: 4px
- **S**: 8px
- **M**: 16px
- **L**: 24px
- **XL**: 32px
- **XXL**: 48px

### Responsive Breakpoints

- **Mobile**: < 640px (single column)
- **Tablet**: 640px - 1024px (2 columns)
- **Desktop**: > 1024px (3 columns)

### Component Shadows

- **Elevation 1**: `0 1px 2px 0 rgba(0,0,0,0.05)`
- **Elevation 2**: `0 4px 6px -1px rgba(0,0,0,0.1)`
- **Elevation 3**: `0 10px 15px -3px rgba(0,0,0,0.1)`

---

## Component Library

### Core Components to Build

#### Layout Components
- `Layout` - Main app wrapper with navigation
- `Sidebar` - Navigation sidebar
- `Header` - Top bar with branding
- `Footer` - Optional footer

#### Form Components
- `Input` - Text input with validation
- `Select` - Dropdown selector
- `Checkbox` - Boolean toggle
- `Radio` - Single selection
- `TextArea` - Multi-line input
- `FormGroup` - Label + input wrapper

#### Data Display
- `Card` - Container component
- `Table` - Data table
- `Grid` - Responsive grid layout
- `List` - Ordered/unordered list
- `Badge` - Status indicator

#### Interaction
- `Button` - Primary, secondary, danger variants
- `Modal` - Dialog/popup
- `Tooltip` - Hover hints
- `Tabs` - Tab navigation
- `Accordion` - Collapsible sections
- `Dropdown` - Context menu

#### Status & Feedback
- `Loading` - Spinner/skeleton
- `Empty` - Empty state
- `Error` - Error message
- `Success` - Success toast
- `Alert` - Warning/info banner

---

## State Management

### Redux Structure

```typescript
// Store slices
store: {
  app: {
    currentPage: 'mission-control' | 'config' | 'onboarding',
    theme: 'light' | 'dark',
    sidebarOpen: boolean,
    // ... UI state
  },
  config: {
    agents: Agent[],
    tools: Tool[],
    platforms: Platform[],
    configVersion: string,
    loading: boolean,
    error?: string,
  },
  coordination: {
    health: AgentHealthRecord[],
    metrics: CoordinationMetrics,
    latestStandup?: StandupResult,
    currentMode: CoordinationMode,
    loading: boolean,
    lastUpdated?: timestamp,
  },
  conversation: {
    sessions: ConversationSession[],
    activeSessionId?: string,
    status: 'idle' | 'creating' | 'created' | 'cancelled',
  },
  websocket: {
    connected: boolean,
    lastMessage?: Event,
    reconnecting: boolean,
    connectionCount: number,
  }
}
```

### Persistence

Use Redux Persist to save to localStorage:
- `app.currentPage` (remember last page)
- `app.theme` (dark mode preference)
- `config.*` (cache config for quick load)

---

## Integration Checklist

### For builder.io (or implementing frontend team)

- [ ] **API Integration**
  - [ ] Implement `useAPI` hook with axios
  - [ ] Add error handling and retry logic
  - [ ] Implement request/response interceptors

- [ ] **WebSocket Integration**
  - [ ] Implement `useWebSocket` hook
  - [ ] Add auto-reconnect logic (exponential backoff)
  - [ ] Dispatch Redux actions on events

- [ ] **State Management**
  - [ ] Configure Redux store
  - [ ] Set up Redux Persist
  - [ ] Create all slices (app, config, coordination, conversation, websocket)

- [ ] **Pages**
  - [ ] Welcome page
  - [ ] Onboarding wizard (4 steps)
  - [ ] Configuration dashboard
  - [ ] Mission control dashboard
  - [ ] Layout shell (navigation, sidebar)

- [ ] **Components**
  - [ ] All layout components
  - [ ] All form components
  - [ ] All data display components
  - [ ] All interaction components
  - [ ] All status/feedback components

- [ ] **Styling**
  - [ ] Configure Tailwind CSS
  - [ ] Create design tokens (colors, spacing, typography)
  - [ ] Implement responsive design system
  - [ ] Create component variants (sizes, states)

- [ ] **Testing**
  - [ ] Unit tests for hooks
  - [ ] Integration tests for components
  - [ ] E2E tests for workflows (onboarding, config, monitoring)

- [ ] **Performance**
  - [ ] Implement code splitting (route-based)
  - [ ] Lazy load components
  - [ ] Memoize expensive computations
  - [ ] Optimize WebSocket updates

- [ ] **Accessibility**
  - [ ] ARIA labels on all interactive elements
  - [ ] Keyboard navigation support
  - [ ] Color contrast compliance (WCAG AA)
  - [ ] Screen reader testing

- [ ] **Documentation**
  - [ ] Component Storybook setup
  - [ ] API client documentation
  - [ ] State management guide
  - [ ] Deployment instructions

---

## API Integration Examples

### useAPI Hook

```typescript
// Usage
const { agents, loading, error } = useAPI('/api/config/agents');

// Implementation
function useAPI(endpoint: string) {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetch(endpoint)
      .then(r => r.json())
      .then(d => setData(d))
      .catch(e => setError(e))
      .finally(() => setLoading(false));
  }, [endpoint]);

  return { data, loading, error };
}
```

### useWebSocket Hook

```typescript
// Usage
useWebSocket('ws://localhost:7777/ws', (event) => {
  dispatch(handleCoordinationEvent(event));
});

// Implementation
function useWebSocket(url: string, onMessage: (msg: any) => void) {
  useEffect(() => {
    const ws = new WebSocket(url);
    ws.onmessage = (event) => {
      onMessage(JSON.parse(event.data));
    };
    return () => ws.close();
  }, [url]);
}
```

### Redux Dispatch Example

```typescript
// On page mount
useEffect(() => {
  dispatch(fetchConfig());      // GET /api/config/agents
  dispatch(fetchCoordination()); // GET /api/coordination/*
  dispatch(connectWebSocket()); // ws://localhost:7777/ws
}, [dispatch]);
```

---

## Responsive Design

### Mobile (< 640px)
- Single column layout
- Full-width cards
- Stacked forms
- Hamburger menu

### Tablet (640px - 1024px)
- 2 column grid
- Condensed sidebars
- Optimized touch targets (44px minimum)

### Desktop (> 1024px)
- 3 column grid
- Full navigation
- Hover states on interactive elements

---

## Success Criteria for builder.io

✅ **Functional:**
- Onboarding wizard leads to working dashboard
- Mission Control receives real-time WebSocket events
- Configuration page fetches agents/tools correctly
- All forms validate and submit successfully

✅ **Visual:**
- Design system applied consistently
- Responsive on mobile/tablet/desktop
- Dark mode support (if implemented)
- Animations are smooth (60fps)

✅ **Performance:**
- Page load < 2 seconds
- WebSocket events processed within 100ms
- Redux state updates don't cause excessive re-renders
- Memory usage stable over time

✅ **Accessibility:**
- WCAG AA compliance
- Keyboard navigation works
- Screen reader compatible
- Color contrast ratio 4.5:1+

---

**Ready for builder.io implementation!** Use this specification as your complete contract. All APIs, components, and workflows are defined. Questions? Review the COMPLETE-API-SPECIFICATION.md for endpoint details.
