---
sidebar_label: Overview
sidebar_position: 1
---

# Architecture Overview

AOF's architecture is designed around composability, safety, and multi-model intelligence. This section covers the core architectural concepts that power the framework.

## Core Principles

1. **Composability**: Build complex workflows from simple, reusable components
2. **Safety**: Context injection ensures environment boundaries are respected
3. **Intelligence**: Multi-model consensus improves accuracy and reduces errors
4. **Scalability**: Multi-tenant routing supports enterprise deployments

## Architecture Documents

### [Composable Design](./composable-design.md)

The foundational architecture of AOF, organized into four layers:

```
Layer 4: TRIGGERS  ─ Platform routing (Slack, Discord, Teams, etc.)
Layer 3: FLOWS     ─ Multi-step workflows with orchestration
Layer 2: FLEETS    ─ Agent composition for collaboration
Layer 1: AGENTS    ─ Single-purpose AI specialists
```

Each layer builds on the previous, enabling you to start simple and add complexity as needed.

### [Context Injection](./context-injection.md)

How AOF ensures the same agent can operate safely across different environments:

- **Environment boundaries**: Isolate production from staging
- **Approval workflows**: Require human approval for destructive operations
- **Rate limiting**: Prevent runaway agents
- **Audit trails**: Track all agent actions for compliance

### [Multi-Model Consensus](./multi-model-consensus.md)

Leverage multiple AI models to improve accuracy:

- **Cross-validation**: Multiple models verify each other's conclusions
- **Weighted voting**: Assign different weights to model opinions
- **Confidence scoring**: Know when to trust the output
- **Fault tolerance**: One wrong model gets outvoted

### [Multi-Tenant Flows](./multi-tenant-agentflows.md)

Scale AOF across organizations, teams, and projects:

- **Platform routing**: Different platforms → different agents
- **Channel isolation**: Team-specific agent configurations
- **User/role matching**: Admin vs developer permissions
- **Organization boundaries**: Enterprise multi-org support

## Visual Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AOF Architecture                                │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         TRIGGER LAYER                                  │  │
│  │  Slack  │  Discord  │  Teams  │  Telegram  │  WhatsApp  │  GitHub    │  │
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│                                    ▼                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                          FLOW LAYER                                    │  │
│  │  Multi-step workflows with nodes, conditions, and approval gates      │  │
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│                                    ▼                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         FLEET LAYER                                    │  │
│  │  Agent composition: collectors → analyzers → synthesizers             │  │
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│                                    ▼                                         │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         AGENT LAYER                                    │  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐         │  │
│  │  │ k8s-ops │ │ aws-ops │ │ docker  │ │terraform│ │  git    │  ...    │  │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘         │  │
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                       CROSS-CUTTING CONCERNS                           │  │
│  │  Context Injection │ Multi-Model Consensus │ Multi-Tenant Routing     │  │
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

## Getting Started

- **New to AOF?** Start with [Composable Design](./composable-design.md) to understand the core concepts
- **Deploying to production?** Read [Context Injection](./context-injection.md) for safety best practices
- **Building RCA workflows?** See [Multi-Model Consensus](./multi-model-consensus.md)
- **Enterprise deployment?** Check [Multi-Tenant Flows](./multi-tenant-agentflows.md)
