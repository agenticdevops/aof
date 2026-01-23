---
sidebar_position: 10
sidebar_label: Kubernetes
---

# Kubernetes MCP Server

Query and manage Kubernetes clusters through kubectl and the Kubernetes API.

## Installation

```bash
# Using npx (recommended)
npx -y @anthropic/mcp-server-kubernetes

# Or install globally
npm install -g @anthropic/mcp-server-kubernetes
```

## Configuration

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: k8s-agent
spec:
  model: google:gemini-2.5-flash
  mcp_servers:
    - name: kubernetes
      command: npx
      args: ["-y", "@anthropic/mcp-server-kubernetes"]
      env:
        KUBECONFIG: ${KUBECONFIG}  # Optional, uses default if not set
```

### Multi-Cluster Configuration

```yaml
mcp_servers:
  - name: k8s-prod
    command: npx
    args: ["-y", "@anthropic/mcp-server-kubernetes"]
    env:
      KUBECONFIG: /path/to/prod-kubeconfig
  - name: k8s-staging
    command: npx
    args: ["-y", "@anthropic/mcp-server-kubernetes"]
    env:
      KUBECONFIG: /path/to/staging-kubeconfig
```

## Available Tools

### kubectl

Execute any kubectl command with full output parsing.

```json
{
  "name": "kubectl",
  "arguments": {
    "command": "get pods -n default -o json"
  }
}
```

**Parameters**:
- `command` (required): kubectl command to execute (without `kubectl` prefix)

**Examples**:
```yaml
# Get all pods in a namespace
command: "get pods -n production"

# Describe a deployment
command: "describe deployment/nginx -n default"

# Get events sorted by time
command: "get events --sort-by='.lastTimestamp'"

# Apply a manifest
command: "apply -f /path/to/manifest.yaml"

# Scale a deployment
command: "scale deployment/web --replicas=3"
```

### get_namespaces

List all namespaces in the cluster.

```json
{
  "name": "get_namespaces",
  "arguments": {}
}
```

### get_pods

List pods with filtering options.

```json
{
  "name": "get_pods",
  "arguments": {
    "namespace": "default",
    "label_selector": "app=nginx",
    "field_selector": "status.phase=Running"
  }
}
```

**Parameters**:
- `namespace` (optional): Namespace to query (default: all namespaces)
- `label_selector` (optional): Label selector (e.g., `app=nginx,env=prod`)
- `field_selector` (optional): Field selector (e.g., `status.phase=Running`)

### get_logs

Get logs from a pod or container.

```json
{
  "name": "get_logs",
  "arguments": {
    "pod": "nginx-abc123",
    "namespace": "default",
    "container": "nginx",
    "tail": 100,
    "since": "1h"
  }
}
```

**Parameters**:
- `pod` (required): Pod name
- `namespace` (optional): Namespace (default: default)
- `container` (optional): Container name (for multi-container pods)
- `tail` (optional): Number of lines from end
- `since` (optional): Duration (e.g., `1h`, `30m`, `2h30m`)
- `previous` (optional): Get logs from previous container instance

### describe_resource

Get detailed information about a resource.

```json
{
  "name": "describe_resource",
  "arguments": {
    "kind": "deployment",
    "name": "nginx",
    "namespace": "default"
  }
}
```

**Parameters**:
- `kind` (required): Resource kind (pod, deployment, service, etc.)
- `name` (required): Resource name
- `namespace` (optional): Namespace

### get_events

Get events for troubleshooting.

```json
{
  "name": "get_events",
  "arguments": {
    "namespace": "default",
    "involved_object": "pod/nginx-abc123",
    "types": ["Warning"]
  }
}
```

**Parameters**:
- `namespace` (optional): Namespace filter
- `involved_object` (optional): Filter by involved object
- `types` (optional): Event types (`Normal`, `Warning`)

## Use Cases

### Pod Debugging Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: pod-debugger
spec:
  model: google:gemini-2.5-flash
  instructions: |
    You are a Kubernetes pod debugging specialist.

    When asked about pod issues:
    1. Get pod status and events
    2. Check logs for errors
    3. Describe the pod for configuration issues
    4. Suggest remediation steps
  mcp_servers:
    - name: kubernetes
      command: npx
      args: ["-y", "@anthropic/mcp-server-kubernetes"]
```

### Deployment Status Checker

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: deploy-checker
spec:
  model: google:gemini-2.5-flash
  instructions: |
    Check deployment status and report:
    - Replica status
    - Pod health
    - Recent events
    - Resource usage
  mcp_servers:
    - name: kubernetes
      command: npx
      args: ["-y", "@anthropic/mcp-server-kubernetes"]
```

## Security Considerations

1. **RBAC**: Use service accounts with minimal required permissions
2. **Namespace Isolation**: Restrict agents to specific namespaces
3. **Audit Logging**: Enable Kubernetes audit logs for MCP actions
4. **Read-Only Mode**: Use read-only service accounts for monitoring agents

### Example RBAC

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: aof-agent-readonly
  namespace: production
rules:
  - apiGroups: [""]
    resources: ["pods", "pods/log", "services", "events"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["apps"]
    resources: ["deployments", "replicasets"]
    verbs: ["get", "list", "watch"]
```

## Troubleshooting

### Connection Issues

```bash
# Verify kubeconfig
kubectl config current-context
kubectl cluster-info

# Test MCP server directly
npx -y @anthropic/mcp-server-kubernetes
```

### Permission Errors

Check service account permissions:
```bash
kubectl auth can-i get pods --as=system:serviceaccount:default:aof-agent
```

## Related

- [Pod Debugger Agent](/docs/agent-library/kubernetes/pod-debugger)
- [Deployment Guardian Agent](/docs/agent-library/kubernetes/deploy-guardian)
- [Kubernetes Triggers](/docs/triggers/kubernetes)
