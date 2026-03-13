# Quickstart: Agent Coordination with Memory

## Overview

This guide demonstrates the **coordinator-specialist** pattern with vector memory. You will:

1. Start the OpenAgentiX gateway
2. Register a `dba-specialist` agent (specialist with memory and research phase enabled)
3. Register an `ops-coordinator` agent (coordinator that delegates to the specialist)
4. Run the coordinator with a database analysis task
5. Watch the delegation flow and audit trail
6. Inspect and re-use accumulated memory

## Prerequisites

- `agentix` CLI installed (`curl -sSL https://docs.aof.sh/install.sh | bash`)
- An API key for Anthropic or your preferred LLM provider
- The `quickstart/` directory from the OpenAgentiX repository

## Setup

Set your API key:

```bash
export ANTHROPIC_API_KEY=sk-...
```

## Step 1: Start the Gateway

```bash
agentix gateway start --config quickstart/serve-config.yaml
```

The gateway starts on `http://localhost:7777`. Leave this running in a terminal window.

## Step 2: Register the Specialist Agent

The `dba-specialist` agent handles PostgreSQL analysis tasks. It has memory enabled (stores past analyses) and the research phase enabled (gathers context before responding).

```bash
agentix apply -f quickstart/agents/dba-specialist.yaml
```

Expected output:

```
agent "dba-specialist" registered
```

## Step 3: Register the Coordinator Agent

The `ops-coordinator` agent analyzes incoming tasks and delegates sub-tasks to specialists via the `/delegate` HTTP endpoint.

```bash
agentix apply -f quickstart/agents/ops-coordinator.yaml
```

Expected output:

```
agent "ops-coordinator" registered
```

## Step 4: Run the Coordinator

```bash
agentix run ops-coordinator \
  --input "Analyze replication lag on pg-primary and provide tuning recommendations"
```

**What happens:**

1. The `ops-coordinator` receives the input and identifies it as a database task
2. It calls the `delegate_to_dba` tool, which POSTs to `/api/v1/agents/dba-specialist/delegate`
3. The gateway fires the `dba-specialist` agent with the delegated task
4. The `dba-specialist` runs its research phase (pre-loop fact gathering), then executes the main ReAct loop
5. The specialist's final answer is returned to the coordinator as a tool result
6. The coordinator synthesizes the result and outputs an executive summary

## Step 5: View the Audit Trail

Every run — including delegations — is recorded:

```bash
agentix runs
```

You will see entries for both agents, with the delegation linked by `delegation_id` in the metadata.

## Step 6: Inspect Memory

After the first run, both agents have stored their outputs in their SQLite memory databases:

```bash
# View dba-specialist memory entries
agentix memory list dba-specialist

# View ops-coordinator memory entries
agentix memory list ops-coordinator
```

Example output for `dba-specialist`:

```
ID                                     RUN_ID       TEXT                                               STORED_AT
------------------------------------------------------------------------------------------------------------------------
3a2b1c0d-...                           run-abc      Replication lag is 2ms — within normal range. ...  2026-03-13T10:30:00Z
```

## Step 7: Run Again — Memory in Action

Run the coordinator a second time with a related query:

```bash
agentix run ops-coordinator \
  --input "Is the replication lag still healthy on pg-primary?"
```

This time, both agents automatically recall relevant context from their first run. The `dba-specialist` prepends the previous analysis to its system prompt as "## Relevant context from past runs", enabling more informed responses without repeating the full analysis.

## Cleaning Up Memory

To reset an agent's memory:

```bash
agentix memory clear dba-specialist --yes
agentix memory clear ops-coordinator --yes
```

## Architecture Summary

```
User Input
    |
ops-coordinator (CoAct loop)
    |
    +-- delegate_to_dba tool ──> POST /api/v1/agents/dba-specialist/delegate
                                           |
                                    dba-specialist
                                    (research phase → ReAct loop → memory store)
                                           |
                                    DelegationResult
    |
ops-coordinator synthesizes result
    |
Final Output
```

## REST API Reference

For programmatic access:

```bash
# Trigger the coordinator
curl -X POST http://localhost:7777/api/v1/agents/ops-coordinator/run \
  -H "Content-Type: application/json" \
  -d '{"input": "Analyze pg-primary replication lag"}'

# Delegate directly to specialist
curl -X POST http://localhost:7777/api/v1/agents/dba-specialist/delegate \
  -H "Content-Type: application/json" \
  -d '{"from_agent": "my-script", "task": "Check pg-primary replication lag", "payload": {"host": "pg-primary"}}'

# List specialist memory
curl http://localhost:7777/api/v1/agents/dba-specialist/memory

# Clear specialist memory
curl -X DELETE http://localhost:7777/api/v1/agents/dba-specialist/memory
```

## Next Steps

- Add more specialist agents (`infra-specialist`, `k8s-specialist`) and wire them into the coordinator's system prompt and tool list
- Enable the research phase on the coordinator for even more informed task routing
- Use `agentix memory list` to audit what knowledge has accumulated over time
- See [Agent Memory](agent-memory.md) for full memory configuration reference
- See [Coordinator Agents](coordinator-agents.md) for the full delegation API reference
