---
sidebar_position: 7
sidebar_label: ITSM
---

# ITSM Agents

Production-ready agents for IT Service Management operations including incident management, change management, and CMDB queries.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [servicenow-ops](#servicenow-ops) | Full ITSM operations | servicenow_* |
| [incident-manager](#incident-manager) | Incident lifecycle | servicenow_incident_* |
| [change-coordinator](#change-coordinator) | Change management | servicenow_change_*, servicenow_cmdb_* |

## servicenow-ops

Comprehensive ServiceNow agent for full ITSM operations including incidents, CMDB, and change management.

### Usage

```bash
# Create incident from alert
aofctl run agent library://itsm/servicenow-ops \
  --prompt "Create P1 incident for database connectivity issue affecting prod-db-01"

# Query incidents
aofctl run agent library://itsm/servicenow-ops \
  --prompt "List all P1 incidents assigned to Database Team"

# CMDB lookup
aofctl run agent library://itsm/servicenow-ops \
  --prompt "Find all production servers in the CMDB"
```

### Capabilities

- Create, update, and resolve incidents
- Query incidents with encoded query filters
- Query CMDB Configuration Items by class
- Create change requests with proper risk assessment
- Full incident lifecycle management

### Tools Used

- `servicenow_incident_create` - Create incidents
- `servicenow_incident_query` - Query incidents
- `servicenow_incident_update` - Update incidents
- `servicenow_incident_get` - Get incident details
- `servicenow_cmdb_query` - Query Configuration Items
- `servicenow_change_create` - Create change requests

### Example Output

```markdown
## ServiceNow Incident Created

### Incident Details
- **Number**: INC0012345
- **Priority**: P1 (Critical)
- **State**: New
- **Short Description**: Database connection pool exhausted on prod-db-01

### Assignment
- **Group**: Database Team
- **Assigned To**: (Pending assignment)

### Related CI
- **Name**: prod-db-01
- **Class**: cmdb_ci_database
- **Environment**: Production
- **Criticality**: 1-Critical

### Timeline
| Time | Action |
|------|--------|
| 14:32:00 | Incident created |
| 14:32:01 | CI linked (prod-db-01) |
| 14:32:02 | Notification sent to Database Team |

### Recommended Next Steps

1. Assign incident to on-call DBA
2. Query related incidents: `cmdb_ci=prod-db-01^state!=6`
3. Check CMDB for dependent services
4. Create change request if configuration change needed
```

### Agent Definition

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: servicenow-ops
  labels:
    category: itsm
    platform: servicenow
spec:
  model: google:gemini-2.5-flash
  tools:
    - servicenow_incident_create
    - servicenow_incident_query
    - servicenow_incident_update
    - servicenow_incident_get
    - servicenow_cmdb_query
    - servicenow_change_create
  environment:
    SERVICENOW_INSTANCE_URL: "${SERVICENOW_INSTANCE_URL}"
    SERVICENOW_USERNAME: "${SERVICENOW_USERNAME}"
    SERVICENOW_PASSWORD: "${SERVICENOW_PASSWORD}"
```

> **Tool Reference**: See [ServiceNow Tools](../tools/servicenow.md) for detailed API documentation.

---

## incident-manager

Specialized agent for incident lifecycle management with focus on SLA compliance.

### Usage

```bash
# Triage incidents
aofctl run agent library://itsm/incident-manager \
  --prompt "Triage unassigned P1/P2 incidents"

# Update with work notes
aofctl run agent library://itsm/incident-manager \
  --prompt "Update INC0012345 with investigation findings"

# Resolve incident
aofctl run agent library://itsm/incident-manager \
  --prompt "Resolve INC0012345 - root cause was connection pool size"
```

### Capabilities

- Incident triage and prioritization
- SLA tracking and escalation
- Work note documentation
- Resolution and closure
- Related incident correlation

### Example Output

```markdown
## Incident Triage Report

### Unassigned High Priority Incidents

| Number | Priority | Age | SLA Status | Description |
|--------|----------|-----|------------|-------------|
| INC0012345 | P1 | 15min | AT RISK | Database connectivity |
| INC0012346 | P2 | 45min | OK | Slow API response |
| INC0012347 | P2 | 2hr | BREACHED | Login failures |

### Recommended Assignments

1. **INC0012345** → Database Team (SLA breach in 15min)
2. **INC0012347** → Security Team (Already breached, needs escalation)
3. **INC0012346** → Platform Team (Within SLA)

### SLA Summary
- P1 Response SLA: 15 minutes
- P2 Response SLA: 1 hour
- Currently at risk: 1
- Currently breached: 1
```

### Agent Definition

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: incident-manager
  labels:
    category: itsm
    focus: incidents
spec:
  model: google:gemini-2.5-flash
  tools:
    - servicenow_incident_query
    - servicenow_incident_update
    - servicenow_incident_get
  environment:
    SERVICENOW_INSTANCE_URL: "${SERVICENOW_INSTANCE_URL}"
    SERVICENOW_USERNAME: "${SERVICENOW_USERNAME}"
    SERVICENOW_PASSWORD: "${SERVICENOW_PASSWORD}"
```

---

## change-coordinator

Specialized agent for change management and CMDB operations.

### Usage

```bash
# Create change request
aofctl run agent library://itsm/change-coordinator \
  --prompt "Create change request for increasing database connection pool"

# Impact analysis
aofctl run agent library://itsm/change-coordinator \
  --prompt "What services depend on prod-db-01?"

# Change assessment
aofctl run agent library://itsm/change-coordinator \
  --prompt "Assess risk for deploying API v2.0 to production"
```

### Capabilities

- Create and manage change requests
- CMDB impact analysis
- Risk assessment
- Change scheduling
- Dependency mapping

### Example Output

```markdown
## Change Request Created

### Change Details
- **Number**: CHG0005678
- **Type**: Normal
- **Risk**: Moderate
- **Short Description**: Increase database connection pool on prod-db-01

### Schedule
- **Planned Start**: 2025-01-21 02:00 UTC
- **Planned End**: 2025-01-21 03:00 UTC
- **Maintenance Window**: Yes

### Impact Analysis

**Affected CI**: prod-db-01

**Dependent Services** (from CMDB):
| Service | Criticality | Impact |
|---------|-------------|--------|
| api-service | 1-Critical | Brief connection reset |
| analytics | 2-High | Temporary query delay |
| reporting | 3-Medium | No impact expected |

### Risk Assessment
- **Technical Risk**: Low (standard configuration change)
- **Business Risk**: Moderate (affects production)
- **Rollback Plan**: Revert pool size to 100

### Approval Required
- CAB Review: Yes (Normal change)
- Technical Approval: Database Team Lead
```

### Agent Definition

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: change-coordinator
  labels:
    category: itsm
    focus: changes
spec:
  model: google:gemini-2.5-flash
  tools:
    - servicenow_change_create
    - servicenow_cmdb_query
    - servicenow_incident_query
  environment:
    SERVICENOW_INSTANCE_URL: "${SERVICENOW_INSTANCE_URL}"
    SERVICENOW_USERNAME: "${SERVICENOW_USERNAME}"
    SERVICENOW_PASSWORD: "${SERVICENOW_PASSWORD}"
```

---

## Environment Setup

```bash
# ServiceNow
export SERVICENOW_INSTANCE_URL=https://company.service-now.com
export SERVICENOW_USERNAME=api_user
export SERVICENOW_PASSWORD=your-password

# Or use OAuth 2.0 (recommended for production)
export SERVICENOW_ACCESS_TOKEN=your-oauth-token
```

## Integration with Observability

ITSM agents work well with observability agents for automated incident response:

```yaml
apiVersion: aof.sh/v1alpha1
kind: Fleet
metadata:
  name: alert-to-incident
spec:
  agents:
    - ref: library://observability/newrelic-ops
    - ref: library://itsm/servicenow-ops
  strategy:
    type: sequential
```

### Workflow Example

1. New Relic agent detects incident
2. Queries affected entities and metrics
3. ServiceNow agent creates incident with context
4. Links to affected CMDB CI
5. Assigns to appropriate team

## Next Steps

- [Observability Agents](./observability.md) - Monitor and alert
- [Incident Agents](./incident.md) - RCA and postmortems
- [ServiceNow Tools Reference](../tools/servicenow.md) - Detailed API docs
