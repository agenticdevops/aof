# Persona UI Components (Developer Reference)

Internal documentation for the agent persona display components in Mission Control UI.

## Component Hierarchy

```
AgentGrid (src/components/AgentGrid.tsx)
  +-- useAgentsConfig() -- fetches /api/config/agents
  +-- useConfigVersion() -- polls for config changes
  +-- useAgentIntroduction() -- manages introduction toasts
  +-- AgentCardSkeleton -- loading shimmer
  +-- AgentCard (React.memo) (src/components/AgentCard.tsx)
  |     +-- PersonalityTraits (src/components/PersonalityTraits.tsx)
  |     +-- StatusIndicator (src/components/StatusIndicator.tsx)
  |     +-- CapabilityBoundaries (src/components/CapabilityBoundaries.tsx)
  |     +-- MetricBadge (inline in AgentCard.tsx)
  +-- IntroductionToast (inline in AgentGrid.tsx)
```

## Type Definitions

### Agent (extended in Phase 5)

```typescript
interface Agent {
  id: string;
  name: string;
  role: string;
  personality?: string;
  avatar?: string;
  skills: string[];
  status: AgentStatus;

  // Phase 5 persona fields
  personality_traits?: string[];
  can?: string[];
  cannot?: string[];
  communication_style?: string;
  tone?: string;
  intro_message?: string;
  uptime_percent?: number;
  success_rate?: number;
}
```

All persona fields are optional for backward compatibility. Components render gracefully when fields are absent.

### Redux State (configSlice)

```typescript
interface ConfigState {
  agents: Agent[];
  tools: Tool[];
  configVersion: string;
  introductions: IntroductionMessage[];
  introducedAgentIds: string[];
}
```

Actions:
- `setAgents(Agent[])` -- replace agent list
- `addIntroduction(IntroductionMessage)` -- add intro (deduped by agent_name)
- `consumeIntroduction(agentName)` -- remove after toast displayed
- `clearIntroductions()` -- reset all

## Component Details

### PersonalityTraits

**Props:** `{ traits: string[], className?: string }`

Renders up to 3 traits as colored pill badges. Color mapping:
- Blue: methodical, proactive, detail-oriented, systematic, analytical
- Purple: curious, patient, thorough, persistent, investigative
- Green: calm, decisive, communicative, confident, organized
- Gray: default for unrecognized traits

Traits >20 chars are truncated with ellipsis. Tooltip on hover/focus shows full text.

"+N more" button expands to show all traits.

### CapabilityBoundaries

**Props:** `{ can: string[], cannot: string[], className?: string }`

Collapsible section with header "Capabilities" + chevron icon.
- Returns null if both arrays empty
- CAN items: green bullet list
- CANNOT items: red bullet list
- Side-by-side on desktop (md:grid-cols-2), stacked on mobile
- Keyboard accessible: Enter/Space to toggle
- aria-expanded attribute tracks state

### MetricBadge (inline)

**Props:** `{ label: string, value: number | undefined }`

Color coding: >=95 green, >=80 yellow, >=60 orange, <60 red.
Undefined values show "--" placeholder.
Tooltip: "Based on last 24 hours of operation"

### Introduction Toasts

Hook: `useAgentIntroduction()`

Returns: `{ activeToasts, dismissToast, isSuppressed, toggleSuppression, focusAgent }`

Behavior:
- Subscribes to `configSlice.introductions`
- Creates toast per new introduction (max 3 visible)
- Auto-dismiss after 8 seconds
- Debounce: 1 per agent per second
- Click toast: scroll to agent card + focus
- localStorage key: `aof-suppress-introductions`

### AgentCard (React.memo)

Memoized to prevent unnecessary re-renders. Three-section layout:
1. **Top:** Avatar + name/role/traits + metrics
2. **Middle:** Status dot + skill tags
3. **Bottom:** Expandable capabilities (click propagation stopped)

`data-agent-id` attribute enables `focusAgent()` navigation.

## Testing

Test file: `src/components/__tests__/AgentCard.test.tsx`

22 tests covering:
- Avatar rendering (with and without)
- Trait badge display (0, 1, 3, 5 traits)
- Capabilities expand/collapse
- CAN/CANNOT color coding
- Reliability metrics display
- Skill tags with truncation
- Keyboard accessibility
- Click handler
- Redux introduction events
- Introduction deduplication
- Grid responsive classes

Run: `npx vitest run src/components/__tests__/AgentCard.test.tsx`

## Responsive Breakpoints

| Tailwind Class | Breakpoint | Columns |
|---------------|------------|---------|
| grid-cols-1 | default | 1 |
| md:grid-cols-2 | >= 768px | 2 |
| lg:grid-cols-3 | >= 1024px | 3 |

Gap: `gap-5` (20px)

## Performance Notes

- `AgentCard` wrapped in `React.memo` -- only re-renders when props change
- `useAgentIntroduction` uses refs to avoid re-render cycles
- Event-to-status mapping uses `useMemo` in AgentGrid
- CSS Grid is native, no JS-based layout computation
- Introduction toast debouncing prevents event spam
