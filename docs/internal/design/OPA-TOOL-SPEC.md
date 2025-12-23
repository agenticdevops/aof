# Open Policy Agent (OPA) Tool Specification

## 1. Overview

The OPA Tool provides programmatic access to Open Policy Agent for policy evaluation, data querying, and compliance enforcement. This tool enables AOF agents to evaluate policies against data, enforce compliance rules, and integrate policy decisions into workflows.

### 1.1 Purpose

- **Policy Evaluation**: Evaluate Rego policies against input data
- **Compliance Checking**: Verify infrastructure configurations against policies
- **Decision Logging**: Query and analyze policy decisions
- **Bundle Management**: Check policy bundle status
- **Data Querying**: Query OPA's document store

### 1.2 OPA Capabilities

OPA provides declarative policy enforcement:
- **Rego Language**: Powerful policy language for complex rules
- **Data Integration**: Policies can reference external data
- **Partial Evaluation**: Compile policies for client-side decisions
- **Decision Logs**: Audit trail of all policy decisions
- **Bundles**: Package and distribute policies

### 1.3 Feature Flag

```toml
[features]
security = ["reqwest", "serde_json"]
```

## 2. Tool Operations

### 2.1 opa_eval

Evaluate a policy against input data.

**Purpose**: Make policy decisions for authorization, validation, or compliance.

**Parameters**:
- `endpoint` (required): OPA server URL (e.g., `http://localhost:8181`)
- `path` (required): Policy path (e.g., `data/authz/allow`)
- `input` (required): Input data as JSON object
- `pretty` (optional): Pretty print result, default: false

**Response**:
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

### 2.2 opa_query

Execute an ad-hoc Rego query.

**Purpose**: Run custom queries against OPA's document store.

**Parameters**:
- `endpoint` (required): OPA server URL
- `query` (required): Rego query (e.g., `data.users[_].admin == true`)
- `input` (optional): Input data for the query

**Response**:
```json
{
  "success": true,
  "result": [
    {"admin_users": ["alice", "bob"]}
  ]
}
```

### 2.3 opa_data_get

Get data from OPA's document store.

**Purpose**: Retrieve stored data for inspection or debugging.

**Parameters**:
- `endpoint` (required): OPA server URL
- `path` (required): Data path (e.g., `roles`, `users/alice`)

**Response**:
```json
{
  "success": true,
  "result": {
    "admin": ["alice", "bob"],
    "developer": ["charlie", "dave"]
  }
}
```

### 2.4 opa_data_put

Store data in OPA's document store.

**Purpose**: Update reference data for policies.

**Parameters**:
- `endpoint` (required): OPA server URL
- `path` (required): Data path
- `data` (required): Data to store as JSON

**Response**:
```json
{
  "success": true,
  "stored": true,
  "path": "roles"
}
```

### 2.5 opa_policy_list

List loaded policies.

**Purpose**: See what policies are available.

**Parameters**:
- `endpoint` (required): OPA server URL

**Response**:
```json
{
  "success": true,
  "policies": [
    {
      "id": "authz",
      "path": "authz/authz.rego",
      "raw": "package authz..."
    },
    {
      "id": "kubernetes/admission",
      "path": "kubernetes/admission.rego",
      "raw": "package kubernetes.admission..."
    }
  ]
}
```

### 2.6 opa_policy_put

Upload a new policy.

**Purpose**: Add or update policies dynamically.

**Parameters**:
- `endpoint` (required): OPA server URL
- `policy_id` (required): Policy identifier
- `policy` (required): Rego policy source code

**Response**:
```json
{
  "success": true,
  "uploaded": true,
  "policy_id": "my-policy"
}
```

### 2.7 opa_health

Check OPA server health and status.

**Purpose**: Verify OPA is running and policies are loaded.

**Parameters**:
- `endpoint` (required): OPA server URL
- `bundles` (optional): Include bundle status, default: true
- `plugins` (optional): Include plugin status, default: true

**Response**:
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

## 3. Common Policy Patterns

### 3.1 Kubernetes Admission Control

```rego
package kubernetes.admission

deny[msg] {
    input.request.kind.kind == "Pod"
    container := input.request.object.spec.containers[_]
    not container.securityContext.runAsNonRoot
    msg := sprintf("Container %v must run as non-root", [container.name])
}
```

### 3.2 Terraform Compliance

```rego
package terraform.aws

deny[msg] {
    resource := input.resource_changes[_]
    resource.type == "aws_s3_bucket"
    not resource.change.after.versioning[0].enabled
    msg := sprintf("S3 bucket %v must have versioning enabled", [resource.address])
}
```

### 3.3 API Authorization

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

## 4. Error Handling

```json
{
  "success": false,
  "error": "evaluation error",
  "error_code": "OPA_EVAL_ERROR",
  "details": "1 error occurred: policy.rego:10: var x is unsafe"
}
```

## 5. Example Agent Usage

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

## 6. Integration Patterns

### 6.1 Kubernetes Gatekeeper

OPA can be deployed as Gatekeeper for K8s admission control:

```yaml
# Check if a pod would be admitted
input = {
  "request": {
    "kind": {"kind": "Pod"},
    "object": pod_spec
  }
}
```

### 6.2 Terraform Validation

Validate Terraform plans before apply:

```yaml
# Load terraform plan JSON
input = terraform_plan_json

# Evaluate against policies
opa_eval(path="data/terraform/deny", input=input)
```

### 6.3 API Gateway Authorization

Integrate with API gateways for authz decisions:

```yaml
input = {
  "method": "POST",
  "path": ["api", "users"],
  "user": {"id": "123", "role": "user"}
}
```
