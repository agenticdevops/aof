# Phase 04: Mission Control UI Component Documentation

**Last Updated:** 2024-02-14
**Phase:** 04-mission-control-ui
**Plans:** 04-02 (Agent Visualization & Kanban Board)

---

## Overview

This document provides detailed API documentation for all React components in the Mission Control UI. Components are organized by feature area and include prop types, event handlers, and usage examples.

---

## Component Hierarchy

```
App (main entry point)
├── AgentGrid
│   └── AgentCard (multiple instances)
│       └── StatusIndicator
├── KanbanBoard
│   ├── Lane (5 instances: backlog, assigned, in-progress, review, done)
│   │   └── TaskCard (multiple instances)
│   ├── KeyboardShortcuts (modal)
│   └── Toast (notifications)
└── Skeleton (loading placeholders)
```

---

## Agent Components

### AgentCard

**File:** `src/components/AgentCard.tsx`

**Purpose:** Display agent information with status indicator and tooltip.

**Props:**

```typescript
interface AgentCardProps {
  agent: Agent; // Agent configuration object
  lastActivity?: string; // ISO 8601 timestamp of last activity
  onClick?: (agentId: string) => void; // Click handler for detail modal
  className?: string; // Optional CSS classes
}
```

**Agent Type:**

```typescript
interface Agent {
  id: string;
  name: string;
  role: string;
  personality?: string;
  avatar?: string; // Emoji or URL
  skills: string[];
  status: 'idle' | 'working' | 'blocked' | 'error';
}
```

**State:**

- `showTooltip: boolean` - Controls tooltip visibility on hover

**Events:**

- `onClick(agentId)` - Triggered when card is clicked or Enter is pressed

**Accessibility:**

- `role="button"` - Semantic role
- `tabIndex={0}` - Keyboard focusable
- `aria-label` - Descriptive label with agent name, role, and status
- `onKeyDown` - Enter/Space support

**Example:**

```tsx
<AgentCard
  agent={{
    id: 'agent-1',
    name: 'K8s Monitor',
    role: 'monitor',
    personality: 'Vigilant and detail-oriented',
    skills: ['kubernetes', 'observability'],
    status: 'working',
  }}
  lastActivity="2024-02-14T12:34:56Z"
  onClick={(id) => console.log('Clicked:', id)}
/>
```

---

### AgentGrid

**File:** `src/components/AgentGrid.tsx`

**Purpose:** Grid layout of agent cards with config polling and real-time status updates.

**Props:**

```typescript
interface AgentGridProps {
  onAgentClick?: (agentId: string) => void; // Click handler passed to AgentCard
  className?: string; // Optional CSS classes
}
```

**State:**

- `previousVersion: string | null` - Tracks config version for change detection
- `showToast: boolean` - Controls toast notification visibility

**Data Sources:**

- `useAgentsConfig()` - Fetches agents from /api/config/agents
- `useConfigVersion()` - Polls /api/config/version every 10s
- `Redux eventsSlice.eventsByAgent` - Maps agent_id to events for status

**Lifecycle:**

1. Mount → Fetch agents
2. Poll version every 10s
3. Version change detected → Refetch agents → Show toast
4. Events arrive → Update agent status dynamically

**Responsive Grid:**

- Mobile (< 640px): 1 column
- Tablet (640-1024px): 2 columns
- Desktop (1024-1280px): 4 columns
- Large desktop (> 1280px): 5 columns

**States:**

- **Loading**: Skeleton placeholders (5 cards)
- **Empty**: "No Agents Configured" message
- **Error**: Error message with retry button
- **Success**: Agent cards with real-time status

**Example:**

```tsx
<AgentGrid onAgentClick={(id) => openAgentDetail(id)} />
```

---

## Kanban Components

### KanbanBoard

**File:** `src/components/KanbanBoard.tsx`

**Purpose:** Main Kanban board with 5 lanes and drag-and-drop functionality.

**Props:**

```typescript
interface KanbanBoardProps {
  className?: string; // Optional CSS classes
}
```

**State:**

- `toast: { message: string; type: 'info' | 'success' | 'error' } | null` - Toast notification state
- `showKeyboardShortcuts: boolean` - Keyboard shortcuts modal visibility

**Data Sources:**

- `useTaskManagement()` - Fetches tasks, handles move operations
- `Redux tasksSlice.optimisticTasks` - Renders optimistic state

**Lanes:**

1. **Backlog** - New tasks
2. **Assigned** - Tasks assigned to agents
3. **In Progress** - Tasks being worked on
4. **Review** - Tasks awaiting review
5. **Done** - Completed tasks

**Drag-and-Drop:**

- **Sensors**: PointerSensor, TouchSensor, KeyboardSensor (via `useDndSensors()`)
- **Collision Detection**: closestCorners
- **handleDragEnd**: Triggers optimistic update → POST /api/tasks/move → commit or rollback

**Workflow:**

1. User drags task from lane A to lane B
2. `updateTaskLaneOptimistic` dispatched (instant visual feedback)
3. POST /api/tasks/move sent with { taskId, newLane, version }
4. On success (200): `commitTaskLaneUpdate` dispatched
5. On conflict (409): `rollbackTaskLaneUpdate` dispatched + toast
6. On error (5xx): Retry with exponential backoff

**Keyboard Shortcuts:**

- `?` key opens KeyboardShortcuts modal

**Accessibility:**

- `aria-live="polite"` region for screen reader announcements
- Toast messages announced to screen readers

**Example:**

```tsx
<KanbanBoard />
```

---

### Lane

**File:** `src/components/Lane.tsx`

**Purpose:** Droppable container for tasks in a single Kanban lane.

**Props:**

```typescript
interface LaneProps {
  laneId: TaskLane; // 'backlog' | 'assigned' | 'in-progress' | 'review' | 'done'
  laneName: string; // Display name
  tasks: Task[]; // Tasks in this lane
  className?: string; // Optional CSS classes
}
```

**TaskLane Type:**

```typescript
type TaskLane = 'backlog' | 'assigned' | 'in-progress' | 'review' | 'done';
```

**DnD Integration:**

- `useDroppable(laneId)` - Makes lane a drop target
- `SortableContext` - Enables task reordering within lane
- `verticalListSortingStrategy` - Vertical sorting

**Visual Feedback:**

- **Normal**: Border color based on lane type
- **Drag Over**: Dashed blue border, background tint

**Header Colors:**

- Backlog: Slate (gray)
- Assigned: Blue
- In Progress: Orange
- Review: Yellow
- Done: Green

**Empty State:**

- Shows "No tasks in {laneName}" with icon when tasks.length === 0

**Layout:**

- **Width**: 280px (fixed)
- **Min Height**: 500px
- **Overflow**: Scrollable (overflow-y-auto)

**Example:**

```tsx
<Lane laneId="in-progress" laneName="In Progress" tasks={tasksInProgress} />
```

---

### TaskCard

**File:** `src/components/TaskCard.tsx`

**Purpose:** Draggable task card for Kanban board.

**Props:**

```typescript
interface TaskCardProps {
  task: Task; // Task data
  className?: string; // Optional CSS classes
}
```

**Task Type:**

```typescript
interface Task {
  id: string;
  title: string;
  description: string;
  lane: TaskLane;
  assignedTo?: string;
  version: number;
  createdAt: string; // ISO 8601
  updatedAt: string; // ISO 8601
  status: TaskStatus;
  priority?: TaskPriority;
  tags?: string[];
  dueDate?: string;
}

type TaskStatus = 'pending' | 'active' | 'blocked' | 'completed' | 'cancelled';
type TaskPriority = 'low' | 'medium' | 'high' | 'critical';
```

**DnD Integration:**

- `useSortable(task.id)` - Makes card draggable
- `transform` - Smooth drag animation
- `transition` - 200ms cubic-bezier
- `isDragging` - Opacity 0.5 + elevated shadow

**Visual Elements:**

- **Border Left**: Color-coded by status
  - Completed: Green
  - Active: Orange
  - Blocked: Red
  - Pending: Gray
- **Drag Handle**: Hamburger menu icon (left side)
- **Title**: Single line, truncated
- **Description**: 2 lines max, truncated
- **Tags**: Max 2 visible, +N for overflow
- **Priority Badge**: Top right (critical/high/medium/low)
- **Status Badge**: Footer (color-coded)
- **Assignee**: Footer (avatar + name or "Unassigned")
- **Version**: Footer (small gray text)

**Accessibility:**

- `role="button"` - Semantic role
- `tabIndex={0}` - Keyboard focusable
- `aria-label` - Task title and lane
- `aria-describedby` - Links to description and status
- Status badge has `aria-label="Status: {status}"`

**Example:**

```tsx
<TaskCard
  task={{
    id: 'task-1',
    title: 'Fix login bug',
    description: 'Users cannot log in with SSO',
    lane: 'in-progress',
    assignedTo: 'agent-1',
    version: 3,
    status: 'active',
    priority: 'high',
    tags: ['bug', 'auth'],
    createdAt: '2024-02-14T10:00:00Z',
    updatedAt: '2024-02-14T12:00:00Z',
  }}
/>
```

---

## Utility Components

### StatusIndicator

**File:** `src/components/StatusIndicator.tsx` (from Phase 04-01)

**Purpose:** Color-coded status indicator for agents and connections.

**Props:**

```typescript
interface StatusIndicatorProps {
  status: 'connected' | 'disconnected' | 'reconnecting' | AgentStatus;
  label?: string; // Optional text label
  className?: string; // Optional CSS classes
}

type AgentStatus = 'idle' | 'working' | 'blocked' | 'error';
```

**Color Mapping:**

- Green: connected, idle
- Yellow: reconnecting, working
- Red: disconnected, blocked, error

**Example:**

```tsx
<StatusIndicator status="working" label="Processing tasks" />
```

---

### Skeleton

**File:** `src/components/Skeleton.tsx`

**Purpose:** Loading placeholder with pulse animation.

**Props:**

```typescript
interface SkeletonProps {
  width?: string; // CSS width (default: '100%')
  height?: string; // CSS height (default: '20px')
  variant?: 'text' | 'circular' | 'rectangular'; // Shape (default: 'rectangular')
  className?: string; // Optional CSS classes
}
```

**Variants:**

- **text**: Rounded corners (for text lines)
- **circular**: Fully rounded (for avatars)
- **rectangular**: Rounded-lg corners (for cards)

**Accessibility:**

- `aria-hidden="true"` - Hidden from screen readers

**Example:**

```tsx
<Skeleton width="200px" height="20px" variant="text" />
<Skeleton width="40px" height="40px" variant="circular" />
<Skeleton width="100%" height="150px" variant="rectangular" />
```

---

### KeyboardShortcuts

**File:** `src/components/KeyboardShortcuts.tsx`

**Purpose:** Help modal documenting keyboard navigation.

**Props:**

```typescript
interface KeyboardShortcutsProps {
  isOpen: boolean; // Modal visibility
  onClose: () => void; // Close handler
}
```

**Shortcuts:**

| Key        | Action                                |
| ---------- | ------------------------------------- |
| Tab        | Navigate between tasks                |
| Space      | Pick up or drop task (drag mode)      |
| Arrow Keys | Move task within lane or between lanes |
| Escape     | Cancel drag operation                 |
| Enter      | Open task details                     |
| ?          | Show keyboard shortcuts modal         |

**Accessibility:**

- `role="dialog"` - Semantic role
- `aria-modal="true"` - Traps focus
- `aria-labelledby` - Links to modal title

**Example:**

```tsx
const [showHelp, setShowHelp] = useState(false);

<KeyboardShortcuts isOpen={showHelp} onClose={() => setShowHelp(false)} />;
```

---

## Custom Hooks

### useTaskManagement

**File:** `src/hooks/useTaskManagement.ts`

**Purpose:** Manages task state with optimistic updates and API integration.

**Returns:**

```typescript
interface UseTaskManagementResult {
  tasks: TasksByLane; // Tasks grouped by lane (optimistic state)
  loading: boolean; // Loading state
  error: string | null; // Error message
  moveTask: (taskId: string, newLane: TaskLane) => Promise<void>; // Move task
  refetchTasks: () => Promise<void>; // Refresh all tasks
}
```

**Features:**

- Optimistic UI updates
- Version-based conflict resolution
- Exponential backoff retry (5xx errors)
- AbortController cleanup on unmount

**API Endpoints:**

- `GET /api/tasks` - Fetch all tasks
- `POST /api/tasks/move` - Move task { taskId, newLane, version }

**Example:**

```tsx
const { tasks, loading, error, moveTask, refetchTasks } = useTaskManagement();

// Move task
await moveTask('task-123', 'in-progress');

// Refresh tasks
await refetchTasks();
```

---

### useDndSensors

**File:** `src/utils/dndConfig.ts`

**Purpose:** Configures dnd-kit sensors for mouse, touch, and keyboard.

**Returns:**

```typescript
Sensor[]; // Array of configured sensors
```

**Configuration:**

- **PointerSensor**: 8px activation distance (prevents accidental drags)
- **TouchSensor**: 250ms hold delay, 5px tolerance
- **KeyboardSensor**: Arrow keys for navigation

**Example:**

```tsx
const sensors = useDndSensors();

<DndContext sensors={sensors}>...</DndContext>;
```

---

## Redux State

### tasksSlice

**File:** `src/store/tasksSlice.ts`

**State Shape:**

```typescript
interface TasksState {
  tasks: TasksByLane; // Server truth
  optimisticTasks: TasksByLane; // UI renders this
  pending: Record<string, PendingRequest>; // In-flight requests
  loading: boolean;
  error: string | null;
}
```

**Actions:**

- `setLoading(boolean)` - Set loading state
- `setError(string | null)` - Set error message
- `setTasks(Task[])` - Batch load tasks
- `updateTaskLaneOptimistic(payload)` - Immediate optimistic update
- `commitTaskLaneUpdate(payload)` - Server confirmed
- `rollbackTaskLaneUpdate(payload)` - Server rejected
- `handleServerTaskUpdate(payload)` - Version-based merge

**Selectors:**

- `selectTasksByLane(state)` - Returns optimistic state (what UI renders)
- `selectTasksForLane(lane)(state)` - Tasks for specific lane
- `selectTaskVersion(taskId)(state)` - Get task version
- `selectPendingCount(state)` - Count of pending requests
- `selectTasksLoading(state)` - Loading state
- `selectTasksError(state)` - Error message

---

## Styling Guidelines

### Tailwind Classes

**Layout:**

- `max-w-7xl mx-auto px-4 py-8` - Container
- `grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4` - Responsive grid
- `flex gap-4 overflow-x-auto` - Horizontal scroll

**Colors:**

- Light theme: `bg-white text-gray-900 border-gray-200`
- Dark theme: `dark:bg-gray-800 dark:text-white dark:border-gray-700`

**Transitions:**

- `transition-all hover:shadow-lg` - Smooth hover effects
- `duration-200` - 200ms transitions
- `ease-in-out` - Easing function

**Accessibility:**

- `:focus-visible` - 2px blue outline, 2px offset
- `sr-only` - Screen reader only content

---

## Testing

### Unit Tests

**File:** `src/store/tasksSlice.test.ts`

**Coverage:**

- Version-based conflict resolution
- Optimistic updates
- Commit/rollback logic

**Run:**

```bash
npm test
```

### Integration Tests

**File:** `src/components/KanbanBoard.test.tsx`

**Coverage:**

- Render all 5 lanes
- Display empty state
- Render tasks in correct lanes
- Loading skeleton
- Keyboard shortcuts button

**Run:**

```bash
npm test
```

---

## Performance

### Bundle Size

**Total increase from Phase 04-01:** < 150KB

- dnd-kit packages: ~80KB
- Tasks slice + components: ~20KB
- Test infrastructure: ~40KB (dev only)

### Optimization Strategies

1. **Lazy Loading**: AgentGrid and KanbanBoard use React.lazy()
2. **Code Splitting**: Vendor chunk (React, Redux)
3. **Memoization**: Selectors use Reselect (built into RTK)
4. **Virtualization**: Not needed (< 500 tasks)

### Rendering Performance

- **Target**: 60fps during drag operations
- **React DevTools Profiler**: Verify only affected tasks/lanes re-render
- **Memory**: 60-80MB on desktop

---

## Accessibility

### WCAG 2.1 AA Compliance

- [x] Keyboard navigation (Tab, Space, Arrow, Escape, Enter)
- [x] Focus indicators (2px blue outline, 2px offset)
- [x] Color contrast (4.5:1 for text, 3:1 for UI components)
- [x] ARIA labels (role, aria-label, aria-describedby)
- [x] Screen reader support (aria-live announcements)

### Screen Reader Testing

- **macOS**: VoiceOver
- **Windows**: NVDA
- **Test**: Task titles, lanes, status, drag actions announced

---

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

---

## Future Enhancements

### Phase 04-03 (Task Detail Modal)

- Task detail modal with timeline
- Edit task form
- Comments section
- Attachment upload

### Phase 04-04 (Real-time Updates)

- WebSocket events for task changes
- Live task creation/deletion
- Multi-user collaboration indicators

---

## References

- **dnd-kit Documentation**: https://docs.dndkit.com
- **Redux Toolkit**: https://redux-toolkit.js.org
- **Tailwind CSS**: https://tailwindcss.com
- **WCAG 2.1**: https://www.w3.org/WAI/WCAG21/quickref/

---

**Document Version:** 1.0
**Last Updated:** 2024-02-14
**Maintained by:** AOF Development Team
