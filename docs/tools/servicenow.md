# ServiceNow Tools

AOF provides native ServiceNow integration tools for managing incidents, querying CMDB configuration items, and creating change requests through the ServiceNow Table API.

> **Feature Flag Required**: These tools require the `itsm` feature flag to be enabled during compilation.

## Prerequisites

- ServiceNow instance (any edition)
- Valid ServiceNow credentials (Basic Auth or OAuth)
- API access enabled on your instance

## Authentication

ServiceNow supports multiple authentication methods:

### Basic Auth (Development)

```yaml
env:
  SERVICENOW_USERNAME: "${SNOW_USER}"
  SERVICENOW_PASSWORD: "${SNOW_PASS}"
```

### OAuth 2.0 (Production Recommended)

```yaml
env:
  SERVICENOW_ACCESS_TOKEN: "${SNOW_TOKEN}"
```

## Instance URL Format

ServiceNow instance URLs follow this pattern:

```
https://{instance}.service-now.com
```

For example:
- `https://company.service-now.com`
- `https://companydev.service-now.com`

## Available Tools

| Tool | Description | Use Cases |
|------|-------------|-----------|
| `servicenow_incident_create` | Create incidents | Automated incident creation |
| `servicenow_incident_query` | Query incidents | Incident search, reporting |
| `servicenow_incident_update` | Update incidents | Status changes, work notes |
| `servicenow_incident_get` | Get incident details | Incident lookup |
| `servicenow_cmdb_query` | Query CMDB | CI discovery, impact analysis |
| `servicenow_change_create` | Create change requests | Change management workflows |

---

## servicenow_incident_create

Create a new incident in ServiceNow for tracking and resolution.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance_url` | string | Yes | ServiceNow instance URL |
| `username` | string | Yes | ServiceNow username |
| `password` | string | Yes | ServiceNow password |
| `short_description` | string | Yes | Brief summary (max 160 chars) |
| `description` | string | No | Detailed description |
| `urgency` | string | No | 1 (High), 2 (Medium), 3 (Low) |
| `impact` | string | No | 1 (High), 2 (Medium), 3 (Low) |
| `category` | string | No | Incident category |
| `subcategory` | string | No | Incident subcategory |
| `assignment_group` | string | No | Assignment group name or sys_id |
| `assigned_to` | string | No | Assigned user name or sys_id |
| `cmdb_ci` | string | No | Configuration Item sys_id |
| `caller_id` | string | No | User who reported the incident |

**Priority Calculation:**

Priority is automatically calculated from Urgency × Impact:

| | Impact 1 | Impact 2 | Impact 3 |
|--------|----------|----------|----------|
| **Urgency 1** | P1 (Critical) | P2 (High) | P3 (Moderate) |
| **Urgency 2** | P2 (High) | P3 (Moderate) | P4 (Low) |
| **Urgency 3** | P3 (Moderate) | P4 (Low) | P5 (Planning) |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: servicenow-incident-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a ServiceNow incident management agent.

    When creating incidents:
    1. Set appropriate urgency and impact
    2. Include detailed description
    3. Assign to correct group
    4. Link to affected CIs

    Incident States:
    - 1: New
    - 2: In Progress
    - 3: On Hold
    - 6: Resolved
    - 7: Closed

  tools:
    - servicenow_incident_create
    - servicenow_incident_query
    - servicenow_incident_update

  env:
    SERVICENOW_INSTANCE_URL: "${SNOW_URL}"
    SERVICENOW_USERNAME: "${SNOW_USER}"
    SERVICENOW_PASSWORD: "${SNOW_PASS}"
```

---

## servicenow_incident_query

Query incidents from ServiceNow with filters and pagination.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance_url` | string | Yes | ServiceNow instance URL |
| `username` | string | Yes | ServiceNow username |
| `password` | string | Yes | ServiceNow password |
| `query` | string | No | Encoded query string |
| `fields` | string | No | Comma-separated fields to return |
| `limit` | integer | No | Max results (default: 50) |
| `offset` | integer | No | Pagination offset (default: 0) |

**Encoded Query Syntax:**

ServiceNow uses encoded queries for filtering:

```
# High priority active incidents
priority=1^state!=6

# Incidents assigned to a group
assignment_group.name=Database Team^state=2

# Created in last 24 hours
sys_created_on>javascript:gs.daysAgo(1)

# Multiple conditions with OR
priority=1^ORpriority=2

# Contains text
short_descriptionLIKEerror
```

**Query Operators:**

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Equals | `priority=1` |
| `!=` | Not equals | `state!=6` |
| `>` | Greater than | `sys_created_on>2025-01-01` |
| `<` | Less than | `priority<3` |
| `LIKE` | Contains | `short_descriptionLIKEerror` |
| `^` | AND | `priority=1^state=2` |
| `^OR` | OR | `priority=1^ORpriority=2` |

---

## servicenow_incident_update

Update an existing incident in ServiceNow.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance_url` | string | Yes | ServiceNow instance URL |
| `username` | string | Yes | ServiceNow username |
| `password` | string | Yes | ServiceNow password |
| `sys_id` | string | Yes | Incident sys_id |
| `fields` | object | Yes | Fields to update |

**Common Update Fields:**

```json
{
  "state": "2",
  "work_notes": "Investigating the issue",
  "assigned_to": "john.doe",
  "close_code": "Solved (Permanently)",
  "close_notes": "Root cause identified and fixed"
}
```

---

## servicenow_incident_get

Get detailed information about a specific incident.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance_url` | string | Yes | ServiceNow instance URL |
| `username` | string | Yes | ServiceNow username |
| `password` | string | Yes | ServiceNow password |
| `identifier` | string | Yes | Incident sys_id or number (e.g., INC0012345) |

---

## servicenow_cmdb_query

Query CMDB Configuration Items for incident context and impact analysis.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance_url` | string | Yes | ServiceNow instance URL |
| `username` | string | Yes | ServiceNow username |
| `password` | string | Yes | ServiceNow password |
| `class` | string | Yes | CI class name |
| `query` | string | No | Encoded query string |
| `fields` | string | No | Comma-separated fields |
| `limit` | integer | No | Max results (default: 50) |

**Common CI Classes:**

| Class | Description |
|-------|-------------|
| `cmdb_ci_server` | Physical and virtual servers |
| `cmdb_ci_database` | Database instances |
| `cmdb_ci_app_server` | Application servers |
| `cmdb_ci_kubernetes_cluster` | Kubernetes clusters |
| `cmdb_ci_cloud_service_account` | Cloud accounts |
| `cmdb_ci_vm_instance` | Virtual machine instances |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: servicenow-cmdb-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a ServiceNow CMDB analysis agent.

    Query the CMDB to:
    1. Find Configuration Items (CIs)
    2. Analyze CI relationships
    3. Identify impact of outages
    4. Correlate incidents with CIs

  tools:
    - servicenow_cmdb_query
    - servicenow_incident_query
```

---

## servicenow_change_create

Create a change request in ServiceNow.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instance_url` | string | Yes | ServiceNow instance URL |
| `username` | string | Yes | ServiceNow username |
| `password` | string | Yes | ServiceNow password |
| `short_description` | string | Yes | Brief change summary |
| `description` | string | No | Detailed description |
| `type` | string | No | Standard, Normal, Emergency |
| `risk` | string | No | High, Moderate, Low |
| `impact` | string | No | 1 (High), 2 (Medium), 3 (Low) |
| `start_date` | string | No | Planned start (ISO 8601) |
| `end_date` | string | No | Planned end (ISO 8601) |
| `cmdb_ci` | string | No | Affected CI sys_id |
| `assignment_group` | string | No | Assignment group |

**Change Types:**

| Type | Description | Approval Required |
|------|-------------|-------------------|
| **Standard** | Pre-approved, low risk | No |
| **Normal** | Regular changes | CAB approval |
| **Emergency** | Urgent changes | Expedited approval |

**Example Agent Configuration:**

```yaml
apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: servicenow-change-agent
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a ServiceNow change management agent.

    Capabilities:
    1. Create change requests for infrastructure changes
    2. Query CMDB for affected CIs
    3. Assess change risk and impact

    Change workflow:
    1. Identify affected CIs
    2. Assess risk and impact
    3. Create change request with details

  tools:
    - servicenow_change_create
    - servicenow_cmdb_query
```

---

## Pre-built Agents

For production-ready agents using these tools, see the Agent Library:

- **[servicenow-ops](../agent-library/itsm.md#servicenow-ops)** - Comprehensive ServiceNow ITSM agent
- **[incident-manager](../agent-library/itsm.md#incident-manager)** - Incident lifecycle management
- **[change-coordinator](../agent-library/itsm.md#change-coordinator)** - Change and CMDB operations

```bash
# Run the pre-built agent
aofctl run agent library://itsm/servicenow-ops \
  --prompt "Create P1 incident for database connectivity issue"
```

## Rate Limits

- **Default**: ~166 concurrent transactions per semaphore
- **Per-user limits**: Configurable per instance
- **HTTP 429**: Returned when limits exceeded
- **Headers**: `X-RateLimit-Remaining`, `X-RateLimit-Reset`

## Best Practices

1. **Use encoded queries**: More efficient than client-side filtering
2. **Limit fields**: Use `sysparm_fields` to reduce payload size
3. **Paginate results**: Use `sysparm_limit` and `sysparm_offset`
4. **Use display values**: `sysparm_display_value=true` for readable output
5. **Cache static data**: CI data, categories, assignment groups

## Security Considerations

1. **Credential Management**: Store credentials in environment variables
2. **OAuth 2.0 for Production**: Recommended over Basic Auth
3. **API Users**: Create dedicated users with minimal permissions
4. **Credential Rotation**: Rotate credentials regularly
5. **ACL Review**: Ensure API access is properly restricted

## See Also

- [ITSM Agents](../agent-library/itsm.md) - Pre-built ServiceNow agents
- [New Relic Tools](./newrelic.md) - Observability integration
- [Splunk Tools](./splunk.md) - SIEM and log analysis
- [Built-in Tools Reference](./builtin-tools.md) - Complete list of built-in tools
