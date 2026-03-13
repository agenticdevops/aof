# Command Center

OpenAgentiX Command Center is a web dashboard for monitoring and managing AI agents running on an agentix gateway. It is a SvelteKit single-page application that communicates directly with the gateway REST API and WebSocket endpoint.

## What It Is

The Command Center is the operational interface for an OpenAgentiX deployment. It gives teams real-time visibility into every agent run, cost, trace, and approval — without needing to use the CLI or read raw JSON.

## Key Features

| Feature | Description |
|---------|-------------|
| **Dashboard** | Metrics bar, real-time activity feed, active runs, pending approvals, scheduled agents |
| **Agent Monitoring** | Live status badges per agent, detail view with config, runs, and costs |
| **Cost Analytics** | Bar, line, and doughnut charts with date range filtering and per-run drill-down |
| **Trace Viewer** | Jaeger-style waterfall timeline for every agent run; multi-agent coordination graph |
| **Approval Queue** | Approve or deny pending actions in real time with history and audit trail |
| **Agent Builder** | Visual form, SOUL.md editor, skill browser, YAML preview, and test run |
| **Settings** | Gateway URL configuration, theme selector, first-run wizard relaunch |
| **First-Run Wizard** | 4-step onboarding: welcome, connect gateway, explore agents, done |

## Architecture

```
Browser (SvelteKit SPA)
        │
        ├── REST API calls ──► agentix gateway :7777
        │                        /api/v1/*
        │                        /healthz
        │
        └── WebSocket ────────► /ws
                                Real-time events:
                                  agent_status, run_event,
                                  approval_event, cost_update
```

The Command Center has no server-side component — it is a fully static SPA served from the `build/` directory. All data comes directly from the agentix gateway you configure.

## Screenshot Overview

### Dashboard
The dashboard shows a top metrics bar (total agents, active runs, daily cost, pending approvals), a real-time activity feed driven by WebSocket events, and quick panels for active runs, pending approvals, and scheduled agents.

### Agent List
Each agent is shown as a card with its name, model, mode, and a live status badge. Clicking an agent opens the detail view with tabs for overview, configuration, run history, and costs.

### Cost Analytics
Bar chart shows daily spend per agent. Line chart shows cumulative spend over time. Doughnut chart breaks down cost by agent. A date range picker narrows the view.

### Trace Viewer
Each run has a waterfall timeline showing spans (run, iteration, LLM call, tool call) with durations and attributes. If multiple agents coordinated, a directed graph visualization shows the coordination structure.

### Approval Queue
Pending approvals are listed with the agent name, requested tool action, and a timestamp. Each row has Approve and Deny buttons with an optional reason input. Completed approvals are in the history section.

### Agent Builder
A two-panel form (visual fields on the left, YAML preview on the right) lets you create or edit agents. A SOUL.md tab provides a Markdown editor for the agent's system prompt. A skill browser lets you add pre-built skill packs.
