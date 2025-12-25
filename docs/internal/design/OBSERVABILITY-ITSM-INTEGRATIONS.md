# Observability & ITSM Platform Integrations

## Overview

This document serves as the master index for AOF integrations with observability and ITSM platforms. These integrations expand AOF's capabilities to interact with enterprise monitoring, logging, and IT service management systems.

## New Integrations (Phase 5)

| Platform | Type | Category | Feature Flag | Status |
|----------|------|----------|--------------|--------|
| **New Relic** | Tool | Observability | `observability` | 🟡 In Progress |
| **Splunk** | Tool | SIEM/Observability | `siem` | 🟡 In Progress |
| **ServiceNow** | Tool | ITSM | `itsm` | 🟡 In Progress |

## Design Specifications

- [NEWRELIC-TOOL-SPEC.md](./NEWRELIC-TOOL-SPEC.md) - New Relic NerdGraph (GraphQL) integration
- [SPLUNK-TOOL-SPEC.md](./SPLUNK-TOOL-SPEC.md) - Splunk REST API and HEC integration
- [SERVICENOW-TOOL-SPEC.md](./SERVICENOW-TOOL-SPEC.md) - ServiceNow Table API integration

## Existing Integrations (Reference)

| Platform | Type | Spec Document | Implementation |
|----------|------|---------------|----------------|
| Datadog | Tool | [DATADOG-TOOL-SPEC.md](./DATADOG-TOOL-SPEC.md) | `crates/aof-tools/src/tools/datadog.rs` |
| Grafana | Tool | [GRAFANA-TOOL-SPEC.md](./GRAFANA-TOOL-SPEC.md) | `crates/aof-tools/src/tools/grafana.rs` |
| PagerDuty | Trigger | [PAGERDUTY-TRIGGER-SPEC.md](./PAGERDUTY-TRIGGER-SPEC.md) | `crates/aof-triggers/src/platforms/pagerduty.rs` |
| OpsGenie | Trigger | [OPSGENIE-TRIGGER-SPEC.md](./OPSGENIE-TRIGGER-SPEC.md) | `crates/aof-triggers/src/platforms/opsgenie.rs` |

## Architecture Overview

### Tool-Based Integrations

New Relic, Splunk, and ServiceNow are implemented as **tools** in the `aof-tools` crate:

```
crates/aof-tools/src/tools/
├── newrelic.rs      # New Relic NerdGraph (GraphQL)
├── splunk.rs        # Splunk REST API + HEC
├── servicenow.rs    # ServiceNow Table API
├── datadog.rs       # Reference implementation
└── mod.rs           # Tool registry
```

### Why Tools (Not Triggers)?

1. **Primary use case is querying**: Agents need to fetch data, not just receive webhooks
2. **API-first integration**: These platforms offer rich APIs for programmatic access
3. **Bidirectional communication**: Agents can both read and write data
4. **Consistent with existing patterns**: Follows Datadog/Grafana tool patterns

### Webhook/Trigger Support (Future)

Webhook receivers for these platforms can be added later if needed:
- New Relic Webhook Destinations
- Splunk Webhook Alert Actions
- ServiceNow Outbound REST Messages

## Implementation Roadmap

### Phase 1: New Relic (Week 1)
- [ ] `newrelic_nrql_query` - Execute NRQL queries
- [ ] `newrelic_alerts_list` - List alert policies
- [ ] `newrelic_incidents_list` - List active incidents
- [ ] `newrelic_entity_search` - Search entities
- [ ] `newrelic_metrics_query` - Query metric timeslices
- [ ] `newrelic_incident_ack` - Acknowledge incidents

### Phase 2: Splunk (Week 2)
- [ ] `splunk_search` - Execute SPL queries
- [ ] `splunk_alerts_list` - List fired alerts
- [ ] `splunk_saved_searches` - List saved searches
- [ ] `splunk_saved_search_run` - Run saved search
- [ ] `splunk_hec_send` - Send events via HEC
- [ ] `splunk_indexes_list` - List indexes

### Phase 3: ServiceNow (Week 3)
- [ ] `servicenow_incident_create` - Create incidents
- [ ] `servicenow_incident_query` - Query incidents
- [ ] `servicenow_incident_update` - Update incidents
- [ ] `servicenow_incident_get` - Get incident details
- [ ] `servicenow_cmdb_query` - Query CMDB CIs
- [ ] `servicenow_change_create` - Create change requests

### Phase 4: Documentation & Testing (Week 4)
- [ ] User documentation for each tool
- [ ] Code examples and tutorials
- [ ] Integration tests
- [ ] Wire into Docusaurus site

## Feature Flags

```toml
# Cargo.toml
[features]
default = []
observability = ["datadog", "grafana", "newrelic"]
siem = ["splunk"]
itsm = ["servicenow"]
all-tools = ["observability", "siem", "itsm", "cloud", "gitops", "security"]
```

## API Comparison

| Capability | New Relic | Splunk | ServiceNow |
|------------|-----------|--------|------------|
| **API Style** | GraphQL (NerdGraph) | REST | REST (Table API) |
| **Auth** | User API Key | Bearer Token | Basic/OAuth |
| **Rate Limits** | 3000/min/account | Deployment-specific | 166 concurrent |
| **Async Operations** | No | Yes (searches) | No |
| **Pagination** | Cursor-based | Offset/Count | Offset/Limit |

## Example Agent YAML

### Multi-Platform Incident Response

```yaml
apiVersion: aof.sh/v1
kind: Agent
metadata:
  name: cross-platform-responder
spec:
  model:
    provider: google
    name: gemini-2.5-flash

  tools:
    # Observability
    - newrelic_nrql_query
    - newrelic_incidents_list
    - splunk_search

    # ITSM
    - servicenow_incident_create
    - servicenow_cmdb_query

  system_prompt: |
    You are a cross-platform incident response agent.

    When investigating an issue:
    1. Query New Relic for APM metrics and errors
    2. Search Splunk logs for related events
    3. Query ServiceNow CMDB for affected CIs
    4. Create ServiceNow incident with context

    Correlate data across platforms for comprehensive analysis.
```

### Tiered Fleet Example

```yaml
apiVersion: aof.sh/v1
kind: Fleet
metadata:
  name: observability-rca-fleet
spec:
  tiers:
    # Tier 1: Data Collectors (cheap, fast)
    - name: collectors
      consensus: first_success
      agents:
        - name: newrelic-collector
          model: google:gemini-2.5-flash
          tools: [newrelic_nrql_query, newrelic_entity_search]

        - name: splunk-collector
          model: google:gemini-2.5-flash
          tools: [splunk_search, splunk_alerts_list]

    # Tier 2: Analyzers (reasoning)
    - name: analyzers
      consensus: weighted_majority
      inputs: [collectors]
      agents:
        - name: rca-analyst
          model: anthropic:claude-sonnet-4-20250514
          weight: 2
          tools: [servicenow_cmdb_query]

    # Tier 3: Reporter (synthesis)
    - name: reporter
      consensus: single
      inputs: [analyzers]
      agents:
        - name: incident-reporter
          model: anthropic:claude-sonnet-4-20250514
          tools: [servicenow_incident_create]
```

## Testing Strategy

### Unit Tests
- Mock HTTP clients for each platform
- Test parameter validation
- Test response parsing
- Test error handling

### Integration Tests
- Use Wiremock for API mocking
- Test authentication flows
- Test rate limit handling
- Test async operation polling (Splunk)

### End-to-End Tests
- Test against sandbox instances (when available)
- Test complete incident response workflows
- Test cross-platform data correlation

## Security Considerations

1. **Credential Storage**: All credentials via environment variables
2. **API Key Rotation**: Support for token refresh (OAuth)
3. **Audit Logging**: Log all API operations
4. **Data Sanitization**: Sanitize user inputs in queries
5. **Network Security**: TLS required, certificate validation

## References

- [New Relic NerdGraph API](https://docs.newrelic.com/docs/apis/nerdgraph/get-started/introduction-new-relic-nerdgraph/)
- [Splunk REST API Reference](https://docs.splunk.com/Documentation/Splunk/latest/RESTREF/RESTprolog)
- [ServiceNow Table API](https://docs.servicenow.com/bundle/tokyo-api-reference/page/integrate/inbound-rest/concept/c_TableAPI.html)

---

**Document Version**: 1.0
**Last Updated**: 2025-12-25
**Author**: AOF Hive Mind Swarm
**Status**: Master Index for Phase 5 Integrations
