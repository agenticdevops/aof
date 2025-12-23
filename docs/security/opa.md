---
sidebar_position: 6
---

# Open Policy Agent (OPA) Tools

AOF integrates with Open Policy Agent for policy evaluation, data querying, and compliance enforcement using Rego policies.

## Available Tools

| Tool | Description |
|------|-------------|
| `opa_eval` | Evaluate a policy against input data |
| `opa_query` | Execute an ad-hoc Rego query |
| `opa_data_get` | Get data from OPA's document store |
| `opa_data_put` | Store data in OPA's document store |
| `opa_policy_list` | List loaded policies |
| `opa_policy_put` | Upload a new policy |
| `opa_health` | Check OPA server health |

## Configuration

Set the OPA server URL:

```bash
export OPA_URL="http://localhost:8181"
```

## Tool Reference

### opa_eval

Evaluate a policy against input data.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |
| `path` | string | Yes | Policy path (e.g., `data/authz/allow`) |
| `input` | object | Yes | Input data as JSON object |
| `pretty` | boolean | No | Pretty print result |

**Example:**

```yaml
# Evaluate authorization policy
tools:
  - opa_eval

# "Check if user alice can read resource /api/users"
```

**Response:**

```json
{
  "success": true,
  "result": true,
  "decision_id": "abc123",
  "metrics": {
    "timer_rego_query_eval_ns": 12345
  }
}
```

### opa_query

Execute an ad-hoc Rego query.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |
| `query` | string | Yes | Rego query |
| `input` | object | No | Input data for the query |

**Example:**

```yaml
# "Find all admin users in the system"
# Query: data.users[_].admin == true
```

**Response:**

```json
{
  "success": true,
  "result": [
    {"admin_users": ["alice", "bob"]}
  ]
}
```

### opa_data_get

Get data from OPA's document store.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |
| `path` | string | Yes | Data path (e.g., `roles`, `users/alice`) |

**Response:**

```json
{
  "success": true,
  "result": {
    "admin": ["alice", "bob"],
    "developer": ["charlie", "dave"]
  }
}
```

### opa_data_put

Store data in OPA's document store.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |
| `path` | string | Yes | Data path |
| `data` | object | Yes | Data to store as JSON |

### opa_policy_list

List loaded policies.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |

**Response:**

```json
{
  "success": true,
  "policies": [
    {
      "id": "authz",
      "path": "authz/authz.rego"
    },
    {
      "id": "kubernetes/admission",
      "path": "kubernetes/admission.rego"
    }
  ]
}
```

### opa_policy_put

Upload a new policy.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |
| `policy_id` | string | Yes | Policy identifier |
| `policy` | string | Yes | Rego policy source code |

### opa_health

Check OPA server health and status.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | OPA server URL |
| `bundles` | boolean | No | Include bundle status (default: true) |
| `plugins` | boolean | No | Include plugin status (default: true) |

**Response:**

```json
{
  "success": true,
  "healthy": true,
  "bundles": {
    "authz-bundle": {
      "status": "ok",
      "last_successful_download": "2024-01-15T10:00:00Z"
    }
  },
  "plugins": {
    "decision_logs": {
      "state": "OK"
    }
  }
}
```

## Common Policy Patterns

### Kubernetes Admission Control

```rego
package kubernetes.admission

deny[msg] {
    input.request.kind.kind == "Pod"
    container := input.request.object.spec.containers[_]
    not container.securityContext.runAsNonRoot
    msg := sprintf("Container %v must run as non-root", [container.name])
}
```

### Terraform Compliance

```rego
package terraform.aws

deny[msg] {
    resource := input.resource_changes[_]
    resource.type == "aws_s3_bucket"
    not resource.change.after.versioning[0].enabled
    msg := sprintf("S3 bucket %v must have versioning enabled", [resource.address])
}
```

### API Authorization

```rego
package authz

default allow = false

allow {
    input.method == "GET"
    input.path == ["api", "public"]
}

allow {
    input.user.role == "admin"
}
```

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: compliance-auditor
spec:
  model: google:gemini-2.5-flash
  tools:
    - opa_eval
    - opa_query
    - opa_data_get
    - opa_policy_list

  environment:
    OPA_URL: "${OPA_URL}"

  system_prompt: |
    You are a compliance auditor using OPA policies.

    ## Capabilities
    - Evaluate resources against compliance policies
    - Query policy data and decisions
    - Explain policy violations
    - Recommend remediation steps

    ## Workflow
    1. Understand what resource/action needs to be evaluated
    2. Use opa_eval to check against relevant policies
    3. If denied, explain why the policy failed
    4. Suggest how to make the resource compliant
```

## Integration Patterns

### Kubernetes Gatekeeper

```yaml
# Check if a pod would be admitted
input = {
  "request": {
    "kind": {"kind": "Pod"},
    "object": pod_spec
  }
}
```

### Terraform Validation

```yaml
# Validate Terraform plans before apply
input = terraform_plan_json
opa_eval(path="data/terraform/deny", input=input)
```

### API Gateway Authorization

```yaml
input = {
  "method": "POST",
  "path": ["api", "users"],
  "user": {"id": "123", "role": "user"}
}
```
