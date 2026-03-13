# Command Center Features

## Dashboard

The dashboard is the home screen of the Command Center.

**Metrics Bar** — Four summary cards at the top:
- Total registered agents
- Currently running agents
- Daily estimated cost in USD
- Pending approval count

**Activity Feed** — Real-time log of WebSocket events from the gateway (agent status changes, run completions, approval decisions, cost updates). Newest events appear at the top; the last 20 events are shown.

**Active Runs Panel** — Lists agents currently executing. Shows agent name and a running badge. Links to the full Runs page.

**Pending Approvals Panel** — Shows how many approval requests are waiting. Links directly to the Approvals page.

**Scheduled Agents Panel** — Lists agents that have cron triggers with a human-readable description of their schedule (e.g., "Every Mon at 09:00").

---

## Agents

**Agent List** — A grid of agent cards. Each card shows:
- Agent name and namespace (if set)
- Model (preferred and fallback)
- Mode (autonomous, semi-autonomous, manual)
- Live status badge: idle / running / scheduled / error

Status badges update in real time via WebSocket events — no page refresh needed.

**Agent Detail** — Clicking an agent opens its detail page at `/agents/:name` with:
- **Overview tab**: Model, mode, skills, tools, triggers, budget configuration
- **Runs tab**: Paginated run history with status, trigger source, duration, and cost
- **Configuration tab**: Full agent configuration as rendered fields

---

## Runs

The Runs page shows a global history of all agent executions across all agents.

**Filtering** — Filter by:
- Agent name (dropdown)
- Status (running, completed, failed)
- Date range

**Run detail** — Each row shows: run ID, agent name, trigger source, start time, duration, status, and cost.

Clicking a run with trace data links to the Trace Viewer.

---

## Costs

The Cost Dashboard provides financial visibility into agent spending.

**Charts (three views)**:
- **Bar chart** — Daily spend per agent, stacked or grouped
- **Line chart** — Cumulative spending trend over the selected date range
- **Doughnut chart** — Cost breakdown by agent as a percentage

**Date Range Filtering** — Select last 7 days, 30 days, or a custom range.

**Agent Drill-down** — Click an agent in any chart to see per-run cost breakdown with token counts and model used.

---

## Traces

The Trace Viewer provides detailed execution visibility for agent runs.

**Waterfall Timeline** — A Jaeger-style span hierarchy showing:
- Run span (outermost)
- Iteration spans (one per ReAct loop iteration)
- LLM call spans with model, input tokens, output tokens, latency
- Tool call spans with tool name, arguments, duration

Spans are color-coded by kind: run (blue), LLM (purple), tool (green), iteration (gray).

**Span Details** — Clicking a span opens a details panel with all span attributes.

**Coordination Graph** — If a run involved multiple agents (via delegation), a directed graph shows which agent delegated to which. The highest out-degree agent is placed at the top as the coordinator.

---

## Approvals

The Approval Queue manages human-in-the-loop decisions for semi-autonomous agents.

**Pending Requests** — Each pending approval shows:
- Agent name
- Requested tool action (name + description)
- Time pending
- Approve and Deny buttons

**Deny Flow** — Clicking Deny reveals an inline text input for an optional reason before confirming. This avoids an extra modal for urgent operational contexts.

**Optimistic UI** — Approve/Deny decisions update immediately in the UI and roll back if the API call fails.

**History** — Completed approvals (approved, denied, expired) are shown in a collapsed section below the pending queue.

**Real-time Updates** — WebSocket events update the queue automatically when new approval requests arrive.

---

## Agent Builder

The Agent Builder provides a visual interface for creating and editing agents.

**Visual Form** — Fields for:
- Agent name and namespace
- Model (preferred and fallback)
- Mode (autonomous / semi-autonomous / manual)
- Max iterations and timeout
- Budget: daily limit USD and max tokens per run
- Triggers: add cron or webhook triggers

**SOUL.md Editor** — A Markdown textarea for the agent's system prompt and identity.

**Skill Browser** — An inline expandable section listing all built-in skill packs (AWS, Kubernetes, Terraform, Docker, Git, Database, Security, Observability). Click to add/remove skills from the agent.

**YAML Preview** — A real-time YAML representation of the agent definition updates as you fill in the form.

**Test Run** — The "Test Run" button sends a configurable prompt to the gateway and streams the response via SSE, showing the agent's ReAct loop output in real time inside the builder.

**Edit Mode** — Opening the builder at `/builder?agent=name` pre-fills all fields from the existing agent definition for editing.

---

## Settings

The Settings page at `/settings` lets you configure Command Center preferences.

**Gateway Connection** — Change the gateway URL and test the connection without navigating away.

**Appearance** — Choose between Light, Dark, or System theme.

**Onboarding** — Re-launch the first-run setup wizard. Useful after pointing the Command Center at a new gateway.

**About** — App version and links to documentation and GitHub.

Settings are persisted to `localStorage` and take effect immediately.

---

## First-Run Wizard

A 4-step onboarding overlay shown on the first visit (before any gateway is configured).

1. **Welcome** — Introduction with a Get Started button.
2. **Connect to Gateway** — URL input with a Test Connection button. Next is disabled until a successful connection is verified.
3. **Explore** — Shows how many agents are registered on the connected gateway with quick links to the Dashboard and Agent Builder.
4. **Done** — Summary of all available sections. Clicking Open Dashboard completes setup and dismisses the wizard.

The wizard can be re-launched at any time from Settings.
