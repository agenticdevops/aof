# Agent Personas in Mission Control UI

AOF agents are more than automation scripts -- they have distinct personalities, communication styles, and capability boundaries defined in workspace configuration files. The Mission Control UI renders these personas visually so operators can understand each agent at a glance.

## How Personas Appear in the UI

### AgentCard Layout

Each agent is displayed as a card in a responsive grid:

```
+----------------------------------------------------+
| [Avatar]  Agent Name            Uptime 98%         |
|           Role Title            Success 96%        |
|           [methodical] [proactive] [detail-oriented]|
|                                                     |
| * idle | kubectl  jq  prometheus  +2                |
|                                                     |
| > Capabilities                                      |
|   I CAN: kubectl operations, pod debugging          |
|   I CANNOT: modify cluster RBAC                     |
+----------------------------------------------------+
```

**Top Section:**
- **Avatar** -- Large emoji (4xl) from `AGENTS.md` `avatar` field
- **Name** -- Agent display name
- **Role** -- Subtitle showing the agent's specialization
- **Traits** -- Colored pill badges showing personality keywords
- **Metrics** -- Uptime and success rate percentages

**Middle Section:**
- **Status indicator** -- Color-coded dot (green=idle, yellow=working, red=error)
- **Skill tags** -- First 3 skills shown, with "+N more" for additional

**Bottom Section:**
- **Capabilities** -- Expandable section showing CAN (green) and CANNOT (red) lists

### Responsive Grid

The agent grid adapts to screen size:

| Breakpoint | Columns | Use Case |
|-----------|---------|----------|
| < 768px   | 1       | Mobile   |
| 768-1024px| 2       | Tablet   |
| > 1024px  | 3       | Desktop  |

### Introduction Toasts

When an agent introduces itself for the first time (e.g., on daemon startup), a toast notification appears:

```
+------------------------------------------+
| [Avatar] Agent Name                   [x] |
| "I'm K8s Monitor, your infrastructure    |
|  specialist. I watch your clusters..."   |
+------------------------------------------+
```

- Toasts auto-dismiss after 8 seconds
- Maximum 3 visible at once (queued overflow)
- Click a toast to navigate to the agent's card
- Dismiss with the X button
- Can be suppressed in settings (persisted to localStorage)

## Configuring Agent Personas

### AGENTS.md

Define agent personas in your workspace `AGENTS.md` file:

```yaml
agents:
  - id: k8s-monitor
    name: Kubernetes Monitor
    role: Infrastructure Specialist
    avatar: "🤖"
    personality_traits:
      - methodical
      - proactive
      - detail-oriented
    can:
      - kubectl operations
      - pod debugging
      - log analysis
      - alerting
    cannot:
      - modify cluster RBAC (too dangerous)
      - delete PVs without approval
    skills:
      - kubectl
      - jq
      - prometheus
    communication_style: calm-professional
    tone: formal
```

### Fields Reference

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier for the agent |
| `name` | Yes | Display name shown in the UI |
| `role` | Yes | Role title (subtitle under name) |
| `avatar` | No | Emoji avatar (falls back to role-based default) |
| `personality_traits` | No | Array of trait keywords (displayed as badges) |
| `can` | No | Actions the agent is authorized to perform |
| `cannot` | No | Actions the agent is restricted from performing |
| `skills` | No | Technical capabilities (shown as tags) |
| `communication_style` | No | How the agent communicates (shown in tooltip) |
| `tone` | No | Voice tone descriptor |
| `intro_message` | No | Text shown in introduction toast |

### Personality Trait Colors

Traits are automatically color-coded by category:

| Category | Traits | Color |
|----------|--------|-------|
| Analytical | methodical, proactive, detail-oriented, systematic | Blue |
| Investigative | curious, patient, thorough, persistent | Purple |
| Leadership | calm, decisive, communicative, confident | Green |
| Other | (any unrecognized trait) | Gray |

### Reliability Metrics

Uptime and success rate are computed from the agent's event history (last 24 hours). Color coding:

| Range | Color | Meaning |
|-------|-------|---------|
| 95-100% | Green | Healthy |
| 80-94% | Yellow | Degraded |
| 60-79% | Orange | Warning |
| < 60% | Red | Critical |

When metrics are unavailable, a "--" placeholder is shown.

## Component Architecture

```
AgentGrid
  +-- AgentCard (React.memo)
  |     +-- PersonalityTraits (trait badges)
  |     +-- StatusIndicator (status dot)
  |     +-- CapabilityBoundaries (expandable CAN/CANNOT)
  |     +-- MetricBadge (uptime/success)
  +-- IntroductionToasts (from useAgentIntroduction hook)
  +-- AgentCardSkeleton (loading state)
```

### Data Flow

1. `useAgentsConfig` hook fetches agents from `/api/config/agents`
2. Agent data (with persona fields) stored in Redux `configSlice`
3. `AgentGrid` merges config with real-time status from `eventsSlice`
4. `AgentCard` renders persona information
5. Introduction events flow through `configSlice.introductions`
6. `useAgentIntroduction` hook manages toast lifecycle

### Key Files

| File | Purpose |
|------|---------|
| `web-ui/src/components/AgentCard.tsx` | Main card component with persona layout |
| `web-ui/src/components/AgentGrid.tsx` | Responsive grid with loading/error states |
| `web-ui/src/components/PersonalityTraits.tsx` | Trait badge rendering |
| `web-ui/src/components/CapabilityBoundaries.tsx` | Expandable CAN/CANNOT section |
| `web-ui/src/hooks/useAgentIntroduction.ts` | Introduction toast hook |
| `web-ui/src/types/events.ts` | Agent type with persona fields |
| `web-ui/src/store/configSlice.ts` | Redux state for agents and introductions |
