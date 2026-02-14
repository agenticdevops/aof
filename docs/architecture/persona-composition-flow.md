# Persona Composition Flow

This document describes the detailed sequence of operations when the persona system processes workspace files into agent behavior.

## Sequence Diagram: Daemon Startup

```
User                aofctl serve        AgentLoader       SoulLoader       PromptComposer      EventBroadcaster
  |                      |                  |                 |                  |                     |
  |-- aofctl serve -->   |                  |                 |                  |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- load_from_file("AGENTS.md") --->|                  |                     |
  |                      |<--- Vec<Agent> --|                 |                  |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- load_from_file("SOUL.md") ------|------>           |                     |
  |                      |<--- HashMap<String, Soul> --------|------<           |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- validate_personas(agents, souls) |                  |                     |
  |                      |<--- Ok(()) ------|                 |                  |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- PromptComposer::new(agents, souls, tools) -------->|                     |
  |                      |<--- composer ----|-----------------|---------<        |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- build_introduction_event_batch(agents, souls) ----->|                     |
  |                      |<--- Vec<CoordinationEvent> -----  |                  |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- for each event: broadcast.send(event) -------------|----->               |
  |                      |                  |                 |                  |                     |
  |                      |  [WebSocket subscribers receive introduction events]  |                     |
  |                      |                  |                 |                  |                     |
  |                      |-- ReliabilityCache::default_capacity() ------------->|                     |
  |                      |                  |                 |                  |                     |
  |                      |  [Start event subscriber: broadcast -> cache]        |                     |
  |                      |                  |                 |                  |                     |
  |                      |  [Start PersonaWatcher for file changes]             |                     |
  |                      |                  |                 |                  |                     |
  |<-- Server ready --   |                  |                 |                  |                     |
```

## Sequence Diagram: System Prompt Composition

```
AgentExecutor       PromptComposer         Agent (data)      Soul (data)       Tool (data)
     |                    |                     |                 |                 |
     |-- compose_system_prompt("k8s-monitor") ->|                 |                 |
     |                    |                     |                 |                 |
     |                    |-- get agent ------->|                 |                 |
     |                    |<--- Agent ----------|                 |                 |
     |                    |                     |                 |                 |
     |                    |-- get soul ---------|------>          |                 |
     |                    |<--- Option<Soul> ---|------<          |                 |
     |                    |                     |                 |                 |
     |                    |  [Layer 1: Base instructions]         |                 |
     |                    |  [Layer 2: Role from Agent]           |                 |
     |                    |  [Layer 3: Personality from Soul]     |                 |
     |                    |  [Layer 4: Communication from Soul]   |                 |
     |                    |  [Layer 5: CAN/CANNOT from Agent]     |                 |
     |                    |  [Layer 6: Tools from skills] --------|------>          |
     |                    |  [Layer 7: Behavioral rules]          |                 |
     |                    |                     |                 |                 |
     |<--- system_prompt -|                     |                 |                 |
     |                    |                     |                 |                 |
     |  [Pass prompt to LLM as system message]  |                 |                 |
```

## Sequence Diagram: File Change Reload

```
Editor              Filesystem           PersonaWatcher    AgentLoader    PromptComposer
  |                     |                      |                |               |
  |-- save SOUL.md ---->|                      |                |               |
  |                     |-- notify event ----->|                |               |
  |                     |                      |                |               |
  |                     |                      |-- debounce(100ms)              |
  |                     |                      |   (coalesce rapid changes)     |
  |                     |                      |                |               |
  |                     |                      |-- reload AGENTS.md ----------->|
  |                     |                      |<-- Vec<Agent> -|               |
  |                     |                      |                |               |
  |                     |                      |-- reload SOUL.md              |
  |                     |                      |<-- HashMap<Soul>              |
  |                     |                      |                |               |
  |                     |                      |-- validate_personas()         |
  |                     |                      |<-- Ok(())                     |
  |                     |                      |                |               |
  |                     |                      |-- PersonaUpdate ──> channel    |
  |                     |                      |                |               |
  |                     |                      |  [Daemon receives PersonaUpdate]
  |                     |                      |  [Recompose prompts]           |
  |                     |                      |  [Re-emit introduction events] |
```

## Sequence Diagram: Metrics Computation

```
AgentExecutor       EventBroadcaster    ReliabilityCache     REST API         UI
     |                    |                    |                  |              |
     |-- emit(Completed)->|                    |                  |              |
     |                    |-- broadcast ------->|                  |              |
     |                    |                    |                  |              |
     |                    |                    |-- store event    |              |
     |                    |                    |-- FIFO eviction  |              |
     |                    |                    |-- recompute      |              |
     |                    |                    |   agent metrics  |              |
     |                    |                    |-- version++      |              |
     |                    |                    |                  |              |
     |                    |                    |                  |              |
     |                    |                    |      GET /api/agents/:id/metrics|
     |                    |                    |<-----|------------|<---poll------|
     |                    |                    |----->|            |              |
     |                    |                    |      |--- JSON --|----->        |
     |                    |                    |      |            |    [render   |
     |                    |                    |      |            |     badge]   |
```

## Component Interactions

```
+------------------+     +------------------+     +------------------+
|  Workspace Files |     |  aof-personas    |     |  aofctl serve    |
|                  |     |                  |     |                  |
|  AGENTS.md ------+---->| AgentLoader      |---->| Config endpoint  |
|  SOUL.md   ------+---->| SoulLoader       |     | /api/config      |
|  TOOLS.md  ------+---->| PromptComposer   |---->| AgentExecutor    |
|                  |     | Events builder   |---->| EventBroadcaster |
|                  |     | ReliabilityCache |---->| Metrics endpoint |
|                  |     | PersonaWatcher   |---->| Reload handler   |
|                  |     | Validation       |     |                  |
+------------------+     +------------------+     +------------------+
                                                         |
                                                         v
                                                  +------------------+
                                                  |  Mission Control |
                                                  |                  |
                                                  |  AgentCard       |
                                                  |  IntroToast      |
                                                  |  MetricBadge     |
                                                  |  EventStream     |
                                                  +------------------+
```
