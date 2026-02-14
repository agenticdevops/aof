# Mission Control Frontend Component Specification

**Version:** 1.0
**Status:** Ready for builder.io
**Last Updated:** 2026-02-14
**Target:** builder.io implementation

---

## Overview

This document specifies the **component architecture**, **prop interfaces**, **behaviors**, and **design patterns** for Mission Control UI. Use this for:
- **builder.io** component configuration
- **Figma/Design reference** system
- **External frontend teams** implementing Mission Control

---

## Architecture Overview

### Component Hierarchy

```
CoordinationPage (Page-level orchestrator)
├── CoordinationStatus (Header: mode + overhead)
├── HeartbeatDashboard (Left panel: agent health grid)
└── StandupFeed (Right panel: daily standup results)
```

### State Management

**Redux Store Structure:**
```typescript
store.coordination = {
  health: AgentHealthRecord[];          // From GET /api/coordination/health
  latestStandup: StandupResult | null;  // From GET /api/coordination/standup/latest
  metrics: CoordinationMetrics | null;  // From GET /api/coordination/metrics
  isLoading: boolean;
  error: string | null;
  coordinationEnabled: boolean;
}
```

**Data Flow:**
1. **Page mount** → fetch health + standup + metrics (REST)
2. **WebSocket connect** → receive real-time updates
3. **Redux dispatch** → components re-render
4. **Metrics polling** → every 30 seconds update overhead

---

## Component: CoordinationStatus

**Purpose:** Display current coordination mode and token overhead gauge.
**Location:** `web-ui/src/components/CoordinationStatus.tsx`
**Placement:** Top of CoordinationPage (full width)

### Props Interface

```typescript
interface CoordinationStatusProps {
  metrics: CoordinationMetrics | null;
  isLoading: boolean;
  onModeChange?: (mode: CoordinationMode) => void;  // Optional override callback
}
```

### Visual Layout

```
┌────────────────────────────────────────────────────────────┐
│ ◆ Full  │ Overhead: 4.2% ████░░░░ │ Token Breakdown        │
│ (mode   │ Target: 30%   ────┬────  │ ┌────────────────────┐│
│  badge) │ (gauge)            │     │ │ Heartbeat:  2.1K  ││
│         │ Colors:            │     │ │ Standup:    2.1K  ││
│         │ Green <20%         │     │ │ Production: 95.8K ││
│         │ Yellow 20-30%      │     │ │ Auto-degrade: ON  ││
│         │ Red >30%           │     │ └────────────────────┘│
│         │                    │     │ [Force Mode ▼]       │
└────────────────────────────────────────────────────────────┘
```

### Features

1. **Mode Badge** (left)
   - Display current mode: Full | Standard | Reduced | HeartbeatOnly | Disabled
   - Colors: Full=green, Standard=blue, Reduced=yellow, HeartbeatOnly=orange, Disabled=gray
   - Clickable to show dropdown

2. **Overhead Gauge** (center)
   - Visual bar: 0-100% width
   - Threshold line at 30% (red line marker)
   - Text percentage below bar
   - Color zones: green <20%, yellow 20-30%, red >30%

3. **Token Breakdown** (right)
   - Show heartbeat tokens
   - Show standup tokens
   - Show production tokens
   - Show auto-degrade status (ON/OFF)

4. **Mode Dropdown** (optional)
   - Click mode badge to show dropdown
   - Options: Full, Standard, Reduced, HeartbeatOnly, Disabled
   - Call `onModeChange(mode)` on selection
   - Close dropdown after selection

### Loading & Error States

- **Loading:** Show skeleton gauge, "Loading..." text
- **Disabled:** Show all as gray, "Coordination disabled"
- **Error:** Show red "Error" badge, disable mode override

### Interactions

| Interaction | Behavior |
|-------------|----------|
| Click mode badge | Show dropdown menu |
| Select mode | POST `/api/coordination/mode`, update UI |
| Hover over gauge | Show tooltip "Coordination overhead vs production work" |
| Refresh metrics | Automatic via 30s polling |

---

## Component: HeartbeatDashboard

**Purpose:** Display grid of agent health cards with live status.
**Location:** `web-ui/src/components/HeartbeatDashboard.tsx`
**Placement:** Left panel (40% width) of CoordinationPage

### Props Interface

```typescript
interface HeartbeatDashboardProps {
  health: AgentHealthRecord[];
  isLoading: boolean;
  onRefresh?: () => void;
}
```

### Visual Layout

```
┌────────────────────────────────┐
│ Agent Health                    │
│ 5/6 healthy | 1 unresponsive   │
│ Last check: 5s ago [↻ Refresh]│
├────────────────────────────────┤
│ ┌─────────┐ ┌─────────┐        │
│ │● k8s    │ │● log    │        │
│ │Healthy  │ │Healthy  │        │
│ │5s ago   │ │8s ago   │        │
│ │145ms    │ │202ms    │        │
│ └─────────┘ └─────────┘        │
│ ┌─────────┐ ┌─────────┐        │
│ │● alert  │ │● db     │        │
│ │Healthy  │ │Healthy  │        │
│ │12s ago  │ │3s ago   │        │
│ │89ms     │ │156ms    │        │
│ └─────────┘ └─────────┘        │
│ ┌─────────┐ ┌─────────┐        │
│ │◯ net    │ │         │        │
│ │UNRESP   │ │         │        │
│ │2m ago   │ │         │        │
│ │3 misses │ │         │        │
│ └─────────┘ └─────────┘        │
└────────────────────────────────┘
```

### Agent Card Design

**Status Colors:**
- 🟢 **Healthy**: Green dot, solid
- 🟡 **Degraded**: Yellow dot, solid
- 🔴 **Unresponsive**: Red dot, **pulsing animation** (draws attention)

**Card Content:**
- Agent ID / name
- Status text
- Last heartbeat (relative: "5s ago", "2m ago")
- Response latency in ms (if available)
- Consecutive misses (if > 0, shown in red)

**Card Styling:**
- Light background (white/light gray)
- Rounded corners (8px)
- Hover: subtle shadow lift
- Unresponsive: 1px red border

### Features

1. **Summary Bar** (top)
   - "X/Y healthy | Z unresponsive | Last check: Ns ago"
   - Refresh button (calls `onRefresh()`)

2. **Grid Layout**
   - 2-3 columns (responsive)
   - Responsive: 3 cols on desktop, 2 on tablet, 1 on mobile

3. **Real-Time Updates**
   - When WebSocket sends `HeartbeatResponse` event:
     - Update agent card with new status + latency
   - When WebSocket sends `HeartbeatTimeout` event:
     - Mark agent unresponsive, increment misses

4. **Empty State**
   - If no agents: "No agents connected"
   - If coordination disabled: "Coordination not enabled. See docs."

5. **Pulsing Animation** (CSS keyframes)
   ```css
   @keyframes pulse {
     0%, 100% { opacity: 1; }
     50% { opacity: 0.5; }
   }
   ```
   Apply to unresponsive agent cards (2s cycle)

### Loading & Error States

- **Loading:** Show 6 skeleton cards
- **Error:** Show error message, disable refresh
- **Empty:** Show helpful message with link to docs

### Interactions

| Interaction | Behavior |
|-------------|----------|
| Click refresh | Call `onRefresh()`, fetch `/api/coordination/health` |
| View card | Show tooltip on hover (agent details) |
| Agent status change (WS) | Animate transition (0.3s fade) |
| Unresponsive agent | Pulse red dot, highlight card border |

---

## Component: StandupFeed

**Purpose:** Display daily standup results in chronological feed with expand/collapse.
**Location:** `web-ui/src/components/StandupFeed.tsx`
**Placement:** Right panel (60% width) of CoordinationPage

### Props Interface

```typescript
interface StandupFeedProps {
  standup: StandupResult | null;
  isLoading: boolean;
  onTrigger?: () => void;  // Callback to trigger manual standup
}
```

### Visual Layout

```
┌──────────────────────────────────────┐
│ Daily Standups                        │
│ [Trigger Standup Now]                │
├──────────────────────────────────────┤
│ February 14, 2026 - 9:00 AM          │
│ ▼ Summary: 2 agents, 1 blocker       │
│                                      │
│ ┌──────────────────────────────────┐ │
│ │ ● k8s-monitor (9:00:15 AM)       │ │
│ │ ┌─────────────────────────────┐  │ │
│ │ │✓ Did: Monitored 12 pods...  │  │ │
│ │ │• Doing: Diagnosing logs...  │  │ │
│ │ │⚠ Blockers: None            │  │ │
│ │ │📊 284 tokens               │  │ │
│ │ └─────────────────────────────┘  │ │
│ └──────────────────────────────────┘ │
│                                      │
│ ┌──────────────────────────────────┐ │
│ │ ● log-analyzer (9:00:30 AM)      │ │
│ │ ┌─────────────────────────────┐  │ │
│ │ │✓ Did: Parsed 45K logs...    │  │ │
│ │ │• Doing: Finding patterns... │  │ │
│ │ │⚠ Blockers: Wait k8s        │  │ │
│ │ │📊 512 tokens               │  │ │
│ │ └─────────────────────────────┘  │ │
│ └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

### Features

1. **Trigger Button** (top)
   - Text: "Trigger Standup Now"
   - Action: `POST /api/coordination/standup/trigger`
   - Disable for 10 seconds after trigger

2. **Date Header**
   - Format: "February 14, 2026 - 9:00 AM"
   - Day-specific grouping (show different header if next day)

3. **Summary** (collapsible)
   - Shows auto-generated summary from Sonnet
   - Click to expand individual agent responses

4. **Agent Response Cards**
   - Each agent that responded gets a card
   - Sections with color coding:
     - ✓ **Did** (green): What they accomplished
     - • **Doing** (blue): Current work
     - ⚠ **Blockers** (red or gray): Blockers or "No blockers"
   - Token count badge (bottom right)
   - Timestamp (human-readable, e.g., "9:00:15 AM")

5. **Progressive Responses**
   - As WebSocket sends `StandupResponse` events:
     - Add card to feed (fade-in animation)
     - Update agent count in summary
   - When `StandupSummary` received:
     - Show summary at top
     - Mark standup as "complete"

6. **Empty State**
   - "No standup results yet. Next standup at 9:00 AM tomorrow."
   - Show trigger button prominently

### Styling Details

**Colors:**
- Did section: Light green background
- Doing section: Light blue background
- Blockers section: Light red (if blockers) or light gray (if none)
- Card background: White with subtle shadow

**Typography:**
- Date header: Bold, 16px
- Agent name: Bold, 14px with colored dot
- Section headers: Semibold, 12px
- Content: Regular, 13px

### Loading & Error States

- **Loading:** Show spinner with "Fetching standup results..."
- **Error:** Show error message, show trigger button to try again
- **Empty:** Show helpful message with trigger button

### Interactions

| Interaction | Behavior |
|-------------|----------|
| Click trigger button | POST `/api/coordination/standup/trigger`, disable for 10s |
| Receive WS event `StandupResponse` | Add agent card with fade-in animation |
| Receive WS event `StandupSummary` | Show summary, mark standup complete |
| Hover on token badge | Show tooltip "Agent used X tokens for this response" |
| Click blocker | Show details/context (if available) |

---

## Component: CoordinationPage

**Purpose:** Orchestrate and layout all coordination components.
**Location:** `web-ui/src/pages/CoordinationPage.tsx`
**Placement:** Top-level page in app router

### Layout

```
┌─────────────────────────────────────────────┐
│ Coordination Overview                        │
│ [CoordinationStatus: full-width bar]         │
├──────────────────┬──────────────────────────┤
│ HeartbeatDash    │ StandupFeed              │
│ (40% width)      │ (60% width)              │
│                  │                          │
│                  │                          │
│                  │                          │
└──────────────────┴──────────────────────────┘
```

### Implementation Pattern

```typescript
// CoordinationPage.tsx
export const CoordinationPage = () => {
  const {
    health,
    latestStandup,
    metrics,
    isLoading,
    error,
    coordinationEnabled,
    triggerStandup,
    forceMode,
    refreshHealth,
  } = useCoordination();

  if (!coordinationEnabled) {
    return <CoordinationDisabledPrompt />;
  }

  return (
    <div className="coordination-page">
      <header>
        <h1>Coordination Overview</h1>
      </header>

      <CoordinationStatus
        metrics={metrics}
        isLoading={isLoading}
        onModeChange={forceMode}
      />

      <div className="dashboard-grid">
        <aside className="left-panel">
          <HeartbeatDashboard
            health={health}
            isLoading={isLoading}
            onRefresh={refreshHealth}
          />
        </aside>

        <main className="right-panel">
          <StandupFeed
            standup={latestStandup}
            isLoading={isLoading}
            onTrigger={triggerStandup}
          />
        </main>
      </div>
    </div>
  );
};
```

### Disabled State

If coordination is disabled, show:

```
┌─────────────────────────────────────────────┐
│ Coordination Not Enabled                    │
│                                             │
│ Mission Control coordination requires setup │
│ configuration.                              │
│                                             │
│ Quick Setup:                                │
│ 1. Set heartbeat_frequency_secs: 30        │
│ 2. Set standup_schedule: "0 9 * * *"       │
│ 3. Restart daemon: aofctl serve --config   │
│                                             │
│ See docs/guides/coordination-setup.md       │
│ [View Setup Guide]                         │
└─────────────────────────────────────────────┘
```

---

## Data Integration Points

### Redux Hooks

**In components, use Redux selectors:**

```typescript
import { useSelector, useDispatch } from 'react-redux';
import { selectHealth, selectLatestStandup, selectMetrics } from '../store/coordinationSlice';

// In component
const health = useSelector(selectHealth);
const latestStandup = useSelector(selectLatestStandup);
const metrics = useSelector(selectMetrics);

// Dispatching
const dispatch = useDispatch();
dispatch(setHealth(newHealthData));
dispatch(setLatestStandup(newStandupData));
dispatch(setMetrics(newMetricsData));
```

### WebSocket Event Mapping

**useWebSocket hook handles these events:**

| WS Event | Redux Action | Component Updated |
|----------|--------------|-------------------|
| `HeartbeatResponse` | `updateAgentHealth()` | HeartbeatDashboard (card animates) |
| `HeartbeatTimeout` | `updateAgentHealth()` with status=Unresponsive | HeartbeatDashboard (card pulses) |
| `StandupResponse` | `addStandupResponse()` | StandupFeed (new card appears) |
| `StandupSummary` | `setLatestStandup()` with summary | StandupFeed (summary shown) |

---

## Design System

### Colors

| Element | Color | Hex |
|---------|-------|-----|
| Healthy status | Green | #10b981 |
| Degraded status | Yellow | #f59e0b |
| Unresponsive status | Red | #ef4444 |
| Full mode | Green | #10b981 |
| Standard mode | Blue | #3b82f6 |
| Reduced mode | Yellow | #f59e0b |
| HeartbeatOnly mode | Orange | #f97316 |
| Disabled mode | Gray | #6b7280 |

### Typography

| Element | Font Size | Weight | Color |
|---------|-----------|--------|-------|
| Page title | 24px | 700 | #1f2937 |
| Section title | 18px | 600 | #374151 |
| Agent name | 14px | 600 | #111827 |
| Body text | 13px | 400 | #4b5563 |
| Token badge | 12px | 500 | #6b7280 |

### Spacing

- Card padding: 16px
- Grid gap: 12px
- Section margin: 24px
- Panel padding: 20px

### Responsive Breakpoints

- Mobile: < 640px (1 column)
- Tablet: 640px - 1024px (2 columns)
- Desktop: > 1024px (3 columns for grid)

---

## Performance Considerations

1. **React.memo** all agent cards (prevent unnecessary re-renders)
2. **useCallback** for event handlers
3. **useMemo** for expensive calculations
4. **Virtual scrolling** if standup feed has many responses
5. **Lazy load** images/avatars

---

## Testing Checklist for builder.io

- [ ] All 3 components render without errors
- [ ] Redux state updates trigger re-renders
- [ ] WebSocket events update UI correctly
- [ ] Loading states show appropriate spinners/skeletons
- [ ] Error states display error messages
- [ ] Responsive layout works on mobile/tablet/desktop
- [ ] Color coding matches spec (green/yellow/red)
- [ ] Pulsing animation works on unresponsive agents
- [ ] Mode dropdown opens/closes correctly
- [ ] Standup trigger button disables for 10s
- [ ] Metrics gauge shows correct colors based on overhead %
- [ ] Agent cards animate on status change

---

## References

- **API Specification:** `docs/api/COORDINATION-API-SPEC.md`
- **Design System:** `docs/concepts/mission-control-coordination.md`
- **Example Dashboards:** FleetControl repositories
- **TypeScript Types:** `web-ui/src/types/coordination.ts`

---

**Ready for builder.io:** All specs complete. Component props, behaviors, and integration points are clearly defined.
