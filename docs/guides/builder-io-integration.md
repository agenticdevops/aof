# Builder.io Integration Guide

## Overview

[Builder.io](https://builder.io) is a visual development platform that enables non-developers to customize UI layouts using drag-and-drop. AOF components can be registered with Builder.io for visual editing of dashboard layouts, agent cards, and other UI elements.

### Use Cases
- Visual editing of dashboard layouts without code changes
- Non-developer customization of agent card appearances
- A/B testing of different UI variants
- Marketing and landing page creation using AOF design system components

## Prerequisites

1. A [Builder.io account](https://builder.io) (free tier available)
2. An API key from the Builder.io dashboard
3. AOF web-app running locally (`cd web-app && pnpm dev`)

## Installation

```bash
cd web-app
pnpm add @builder.io/react
```

## Component Registration

Register AOF components so they appear in the Builder.io visual editor:

### AgentCard

```typescript
import { Builder } from '@builder.io/react'
import AgentCard from '@/components/dashboard/AgentCard'

Builder.registerComponent(AgentCard, {
  name: 'AgentCard',
  inputs: [
    {
      name: 'agent',
      type: 'object',
      required: true,
      subFields: [
        { name: 'id', type: 'string' },
        { name: 'name', type: 'string' },
        { name: 'role', type: 'string' },
        { name: 'status', type: 'string', enum: ['active', 'idle', 'error'] },
        { name: 'personaIcon', type: 'string' },
        { name: 'personaColor', type: 'string' },
      ],
    },
  ],
})
```

### SquadCard

```typescript
import SquadCard from '@/components/fleet/SquadCard'

Builder.registerComponent(SquadCard, {
  name: 'SquadCard',
  inputs: [
    {
      name: 'squad',
      type: 'object',
      required: true,
      subFields: [
        { name: 'id', type: 'string' },
        { name: 'name', type: 'string' },
        { name: 'health', type: 'string', enum: ['healthy', 'degraded', 'critical'] },
        { name: 'memberCount', type: 'number' },
      ],
    },
  ],
})
```

### Badge, Button, EmptyState

```typescript
import { Badge } from '@/components/common/Badge'
import Button from '@/components/common/Button'
import { EmptyState } from '@/components/common/EmptyState'

Builder.registerComponent(Badge, {
  name: 'Badge',
  inputs: [
    { name: 'children', type: 'string', required: true },
    { name: 'variant', type: 'string', enum: ['default', 'success', 'warning', 'error', 'info'] },
  ],
})

Builder.registerComponent(Button, {
  name: 'Button',
  inputs: [
    { name: 'children', type: 'string', required: true },
    { name: 'variant', type: 'string', enum: ['primary', 'secondary', 'ghost', 'danger'] },
    { name: 'size', type: 'string', enum: ['sm', 'md', 'lg'] },
    { name: 'disabled', type: 'boolean' },
  ],
})

Builder.registerComponent(EmptyState, {
  name: 'EmptyState',
  inputs: [
    { name: 'title', type: 'string', required: true },
    { name: 'description', type: 'string' },
  ],
})
```

## Content Integration

Use Builder.io content in your pages:

```typescript
import { BuilderComponent, builder } from '@builder.io/react'

// Initialize with your API key
builder.init('YOUR_BUILDER_IO_API_KEY')

function CustomDashboard() {
  return <BuilderComponent model="page" />
}
```

### Rendering Builder Content in a Route

```typescript
import { BuilderComponent, builder, useIsPreviewing } from '@builder.io/react'

builder.init('YOUR_BUILDER_IO_API_KEY')

function BuilderPage() {
  const isPreviewing = useIsPreviewing()
  const [content, setContent] = React.useState(null)

  React.useEffect(() => {
    builder
      .get('page', { url: window.location.pathname })
      .promise()
      .then(setContent)
  }, [])

  if (!content && !isPreviewing) return null

  return <BuilderComponent model="page" content={content} />
}
```

## Registerable Components

| Component | Location | Builder.io Use |
|-----------|----------|---------------|
| AgentCard | `components/dashboard/AgentCard` | Dashboard layouts |
| SquadCard | `components/fleet/SquadCard` | Fleet overview pages |
| Badge | `components/common/Badge` | Status indicators |
| Button | `components/common/Button` | CTAs and actions |
| EmptyState | `components/common/EmptyState` | Placeholder content |
| Card | `components/common/Card` | Generic container |

## Limitations

1. **Real-time components require Redux**: Components like MessageFeed, AgentGrid, and KanbanBoard depend on the Redux store and WebSocket connection. Use Builder.io only for their layout wrappers, not the interactive content.

2. **WebSocket-connected components must stay React-managed**: The chat feed, live agent status, and task board cannot be fully managed by Builder.io since they rely on real-time data streams.

3. **Dark mode works automatically**: Builder.io content that uses Tailwind CSS classes inherits AOF dark mode support.

4. **Type safety**: Builder.io inputs are loosely typed. Validate props at runtime when using registered components.

## Best Practices

1. **Use Builder.io for static layout sections**: Hero sections, marketing content, configuration pages, and documentation are ideal candidates.

2. **Keep real-time interactive components in React**: Chat feeds, live dashboards, and drag-and-drop boards should remain as coded React components.

3. **Use Builder.io targeting for A/B testing**: Test different dashboard layouts, onboarding flows, or marketing messages without code deploys.

4. **Maintain a registration file**: Create `src/builder-registry.ts` to centralize all `Builder.registerComponent` calls and import it in your app entry point.

5. **Preview mode**: Use `useIsPreviewing()` to show content even when no published content exists, enabling real-time preview in the Builder.io editor.
