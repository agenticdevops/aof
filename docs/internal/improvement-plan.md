# AOF Improvement Plan

> Last Updated: 2026-01-23

## Current State

- **Version**: v0.3.2-beta
- **Tests**: 139 passing
- **Platforms**: Slack, Telegram, Discord, GitHub, Jira, WhatsApp, Bitbucket, GitLab
- **Agents**: 30 pre-built in library
- **Open Issues**: ~20

## Strategic Priorities

### Phase 1: Stabilization & Polish (v0.3.3)

| Priority | Task | Issue | Status |
|----------|------|-------|--------|
| P0 | Close stale issues marked `[DONE]` | #82, #81, #80, #79, #78 | Pending |
| P0 | Update ROADMAP.md (GitHub/Jira implemented) | - | Pending |
| P1 | **Structured I/O Schemas** | #74, #75, #76 | In Progress |
| P1 | **MCP Server Catalog** | #71 | In Progress |
| P2 | Improve error messages with serde_path_to_error | - | Pending |

### Phase 2: Enterprise Features (v0.4.0)

| Priority | Task | Issue | Effort |
|----------|------|-------|--------|
| P1 | **Horizontal scaling** - Redis/NATS message queue | #47 | 1 week |
| P1 | **Multi-org support** - per-org credentials | #46 | 3 days |
| P2 | **ServiceNow trigger** - enterprise ITSM | #48 | 3 days |
| P2 | **Config hot-reload** - no restart updates | #22 | 2 days |

### Phase 3: Observability & Tools (v0.4.x)

| Priority | Task | Issue | Effort |
|----------|------|-------|--------|
| P1 | **Loki tool enhancement** - better log queries | #49 | 2 days |
| P2 | **Jaeger tool** - trace analysis | #50 | 2 days |
| P2 | **Jenkins tool** - CI/CD integration | #55 | 2 days |
| P3 | **NewRelic integration** | - | 3 days |

### Phase 4: Agent Intelligence (v0.5.0)

| Task | Description | Effort |
|------|-------------|--------|
| **Loop node** | Iterate over collections in flows | 2 days |
| **State checkpointing** | Persist flow state for recovery | 3 days |
| **AgentFleet v2** | Better multi-agent coordination | 1 week |
| **Learning/feedback** | Agents learn from outcomes | 2 weeks |

## Developer Experience Focus (Current Priority)

### 1. Structured I/O Schemas

**Goal**: Standardize agent inputs/outputs for better composability.

**Design**:
```yaml
# Agent with structured output
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: pod-analyzer
spec:
  output_schema:
    type: object
    properties:
      status:
        type: string
        enum: [healthy, degraded, critical]
      issues:
        type: array
        items:
          type: object
          properties:
            severity: { type: string }
            message: { type: string }
            recommendation: { type: string }
```

**Benefits**:
- Type-safe flow connections
- Better error handling
- Auto-generated documentation
- IDE autocomplete support

### 2. MCP Server Catalog

**Goal**: Document all available MCP servers and their capabilities.

**Structure**:
```
docs/mcp-catalog/
├── index.md           # Overview and quick reference
├── kubernetes.md      # kubectl, helm, k9s
├── observability.md   # prometheus, grafana, datadog
├── cloud.md           # aws, gcp, azure
├── databases.md       # postgres, redis, mongodb
└── development.md     # git, github, filesystem
```

**Each entry includes**:
- Installation instructions
- Available tools
- Example usage
- Configuration options

## Quick Wins Checklist

- [ ] Close GitHub issues #78, #79, #80, #81, #82
- [ ] Update ROADMAP.md with current status
- [ ] Add `serde_path_to_error` to remaining YAML parsers
- [ ] Add more real-world flow examples
- [ ] Fix GitHub platform test (channel_id format)

## Architecture Decisions

### ADR-001: Structured I/O Schema Format

**Decision**: Use JSON Schema for output definitions, embedded in YAML.

**Rationale**:
- Industry standard
- Tool support (validation, generation)
- Compatible with OpenAPI

### ADR-002: MCP Catalog Organization

**Decision**: Organize by domain (k8s, observability, cloud) not by server.

**Rationale**:
- Users think in terms of what they want to do
- Easier to find relevant tools
- Supports multiple servers per domain

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Test coverage | ~60% | 80% |
| Doc pages | 40+ | 60+ |
| Example flows | 10 | 25 |
| MCP servers documented | 5 | 15 |
| GitHub stars | - | 500 |

## Timeline

```
Week 1-2: Quick wins + Structured I/O design
Week 3-4: Structured I/O implementation + MCP Catalog
Week 5-6: Testing, documentation, v0.3.3 release
Week 7+:  Enterprise features (Phase 2)
```
