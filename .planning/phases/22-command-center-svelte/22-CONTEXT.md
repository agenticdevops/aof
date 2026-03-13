# Phase 22: Command Center (Svelte) - Context

**Gathered:** 2026-03-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Web dashboard (Svelte) to monitor all agents, view run history and costs, inspect execution traces, manage approvals, and build/edit agent definitions — with real-time WebSocket updates and interactive visualizations. Includes a guided first-run wizard for day-0 onboarding.

</domain>

<decisions>
## Implementation Decisions

### Dashboard layout & navigation
- Landing page: Mission control dashboard with key metrics bar up top (total agents, active runs, costs today, pending approvals), recent activity feed, and quick-access panels (active runs, pending approvals)
- Navigation: Collapsible left sidebar with icon-only collapsed state, expanding to labels. Sections: Dashboard, Agents, Runs, Costs, Traces, Approvals, Agent Builder
- Detail views: Slide-over right panel that overlays the current view — keeps list context visible underneath. Dismiss to return
- Visual style: Slate minimal palette (neutral slate grays, subtle card shadows, clean typography). Dark + light modes with toggle

### First-run onboarding
- Guided first-run wizard on initial launch: connect to gateway, register workspace, configure LLM provider keys, create first agent
- Wizard is re-launchable from settings or help menu at any time
- After wizard completion, dashboard shows the newly created agent with empty-state guidance for next steps

### Trace & coordination visualization
- Execution traces: Waterfall timeline (horizontal bars on time axis, like Chrome DevTools / Jaeger). Nested spans indent to show parent-child relationships. Each span shows type, duration, cost
- Multi-agent coordination: Interactive directed graph diagram. Agents as nodes, delegation as directed edges. Click a node to see that agent's trace waterfall
- Charts and visualizations: Must be stunningly beautiful — wow-factor dashboards with polished animations and interactive elements. Cost charts should be a highlight feature

### Agent builder
- Visual form builder: Structured form with fields for name, model selection (dropdown), triggers (add/remove), skills, tools. Generates YAML behind the scenes with inline validation
- SOUL.md editor: Markdown editor with split or tabbed preview — write on one side, rendered preview on the other. Toolbar for formatting
- "Test Run" button: Builder includes a test run button that triggers a single agent run and shows output inline for fast feedback
- Skills: Browsable skill catalog for discovering and attaching skills to agents

### Starting point & tech stack
- Fork & adapt mission-control codebase: Copy into aof repo, strip mission-control-specific features (kanban, chat), keep layout shell, UI components (sidebar, card, badge, button, skeleton), theme system, Tailwind/shadcn setup
- Location: `apps/command-center/` — separate from Rust crates
- API connection: Direct REST + WebSocket to agentix gateway (no SvelteKit server proxy). Gateway already has endpoints from Phases 17-21
- Stack: SvelteKit 5, Tailwind CSS 4, shadcn-svelte, Lucide icons, TypeScript

### Claude's Discretion
- Span detail panel approach (side panel vs inline expandable)
- Chart library selection (must achieve stunning visual quality)
- Skill browser UX within agent builder (modal vs inline)
- Local database decision (localStorage for preferences vs thin SQLite)
- Exact dashboard card layouts and spacing
- Loading states and skeleton patterns
- Error state handling across views

</decisions>

<specifics>
## Specific Ideas

- Mission-control's sidebar, card components, theme toggle, and layout shell should be reused directly — they're proven and match the desired aesthetic
- CalmStudio's Svelte Flow canvas is a reference for the coordination directed graph visualization
- Charts must be a wow-factor feature — "stunningly beautiful dashboards that should wow the users"
- The first-run wizard should feel welcoming and guide a new user from zero to their first working agent
- Wizard must be re-launchable from settings/help at any time

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 22-command-center-svelte*
*Context gathered: 2026-03-13*
