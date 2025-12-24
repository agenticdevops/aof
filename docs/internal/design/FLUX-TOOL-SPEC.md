# Flux Tool Specification

## Overview

The Flux Tool provides comprehensive GitOps capabilities for Kubernetes through the Flux CD toolkit. Flux is a CNCF incubating project that implements continuous delivery for Kubernetes using a Git repository as the source of truth. This tool enables agents to interact with Flux resources for automated deployments, reconciliation, and GitOps workflows.

### Key Capabilities

- **Kustomization Management**: List, inspect, and manage Flux Kustomizations
- **HelmRelease Management**: Manage Helm charts deployed via Flux
- **Reconciliation Control**: Trigger, suspend, and resume resource reconciliation
- **GitOps Workflows**: Integrate with Git-based continuous delivery
- **Multi-tenancy**: Support for namespace-scoped and cluster-wide operations

### Architecture

Flux operates as a set of Kubernetes controllers that:
1. Watch Git repositories for changes
2. Apply Kubernetes manifests automatically
3. Manage Helm releases declaratively
4. Provide image automation capabilities

The Flux Tool interacts with these controllers via the `flux` CLI, which communicates with the Kubernetes API.

## Tool Operations

### 1. flux_kustomization_list

List all Flux Kustomizations in the cluster or namespace.

**Purpose**: Discover all active Kustomizations and their status.

**CLI Command**: `flux get kustomizations`

**Parameters**:
```json
{
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (optional, defaults to all namespaces)",
    "default": ""
  },
  "all_namespaces": {
    "type": "boolean",
    "description": "List Kustomizations from all namespaces",
    "default": true
  },
  "watch": {
    "type": "boolean",
    "description": "Watch for changes (not recommended for agent use)",
    "default": false
  }
}
```

**Output Format**: JSON array of Kustomizations with status, path, revision, and conditions.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "kustomizations": [
      {
        "namespace": "flux-system",
        "name": "flux-system",
        "ready": "True",
        "status": "Applied revision: main/abc123",
        "path": "./clusters/production",
        "revision": "main/abc123"
      }
    ]
  }
}
```

### 2. flux_kustomization_get

Get detailed status of a specific Kustomization.

**Purpose**: Inspect a single Kustomization's configuration, status, and health.

**CLI Command**: `flux get kustomization <name>`

**Parameters**:
```json
{
  "name": {
    "type": "string",
    "description": "Kustomization name (required)"
  },
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (default: flux-system)",
    "default": "flux-system"
  }
}
```

**Output Format**: JSON object with detailed Kustomization status, conditions, and events.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "name": "infrastructure",
    "namespace": "flux-system",
    "ready": true,
    "status": "Applied revision: main/def456",
    "path": "./infrastructure",
    "source": "GitRepository/flux-system/flux-system",
    "revision": "main/def456",
    "conditions": [
      {
        "type": "Ready",
        "status": "True",
        "reason": "ReconciliationSucceeded",
        "message": "Applied revision: main/def456"
      }
    ],
    "last_applied_revision": "main/def456",
    "last_reconcile_time": "2025-12-23T10:30:00Z"
  }
}
```

### 3. flux_helmrelease_list

List all Flux HelmReleases in the cluster or namespace.

**Purpose**: Discover Helm releases managed by Flux.

**CLI Command**: `flux get helmreleases`

**Parameters**:
```json
{
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (optional, defaults to all namespaces)",
    "default": ""
  },
  "all_namespaces": {
    "type": "boolean",
    "description": "List HelmReleases from all namespaces",
    "default": true
  }
}
```

**Output Format**: JSON array of HelmReleases with status, chart version, and conditions.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "helmreleases": [
      {
        "namespace": "applications",
        "name": "nginx-ingress",
        "ready": "True",
        "status": "Release reconciliation succeeded",
        "chart": "nginx-ingress",
        "version": "4.8.3",
        "revision": "1"
      }
    ]
  }
}
```

### 4. flux_helmrelease_get

Get detailed status of a specific HelmRelease.

**Purpose**: Inspect Helm release configuration, values, and deployment status.

**CLI Command**: `flux get helmrelease <name>`

**Parameters**:
```json
{
  "name": {
    "type": "string",
    "description": "HelmRelease name (required)"
  },
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (required)"
  }
}
```

**Output Format**: JSON object with HelmRelease details, status, and Helm revision.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "name": "prometheus",
    "namespace": "monitoring",
    "ready": true,
    "chart": "prometheus-15.0.2",
    "version": "15.0.2",
    "status": "Release reconciliation succeeded",
    "conditions": [
      {
        "type": "Ready",
        "status": "True",
        "reason": "ReconciliationSucceeded"
      }
    ],
    "last_applied_revision": "15.0.2",
    "helm_chart": "monitoring/prometheus"
  }
}
```

### 5. flux_reconcile

Trigger immediate reconciliation of a Flux resource.

**Purpose**: Force Flux to sync changes from Git immediately without waiting for the interval.

**CLI Command**: `flux reconcile <kind> <name>`

**Parameters**:
```json
{
  "kind": {
    "type": "string",
    "description": "Resource kind (kustomization, helmrelease, source, etc.)",
    "enum": ["kustomization", "helmrelease", "source", "gitrepository", "helmrepository", "helmchart"]
  },
  "name": {
    "type": "string",
    "description": "Resource name (required)"
  },
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (default: flux-system)",
    "default": "flux-system"
  },
  "with_source": {
    "type": "boolean",
    "description": "Reconcile the source (GitRepository/HelmRepository) first",
    "default": false
  }
}
```

**Output Format**: Reconciliation result with status and any errors.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "reconciled": true,
    "kind": "kustomization",
    "name": "infrastructure",
    "namespace": "flux-system",
    "message": "reconciliation completed",
    "revision": "main/xyz789"
  }
}
```

### 6. flux_suspend

Suspend reconciliation of a Flux resource.

**Purpose**: Temporarily pause automatic reconciliation (useful during maintenance).

**CLI Command**: `flux suspend <kind> <name>`

**Parameters**:
```json
{
  "kind": {
    "type": "string",
    "description": "Resource kind (kustomization, helmrelease)",
    "enum": ["kustomization", "helmrelease"]
  },
  "name": {
    "type": "string",
    "description": "Resource name (required)"
  },
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (default: flux-system)",
    "default": "flux-system"
  }
}
```

**Output Format**: Suspension confirmation.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "suspended": true,
    "kind": "kustomization",
    "name": "production-apps",
    "namespace": "flux-system",
    "message": "reconciliation suspended"
  }
}
```

### 7. flux_resume

Resume reconciliation of a suspended Flux resource.

**Purpose**: Re-enable automatic reconciliation after suspension.

**CLI Command**: `flux resume <kind> <name>`

**Parameters**:
```json
{
  "kind": {
    "type": "string",
    "description": "Resource kind (kustomization, helmrelease)",
    "enum": ["kustomization", "helmrelease"]
  },
  "name": {
    "type": "string",
    "description": "Resource name (required)"
  },
  "namespace": {
    "type": "string",
    "description": "Kubernetes namespace (default: flux-system)",
    "default": "flux-system"
  }
}
```

**Output Format**: Resume confirmation.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "resumed": true,
    "kind": "helmrelease",
    "name": "nginx-ingress",
    "namespace": "applications",
    "message": "reconciliation resumed"
  }
}
```

### 8. flux_logs

Get logs from Flux controllers.

**Purpose**: Troubleshoot Flux operations by examining controller logs.

**CLI Command**: `flux logs`

**Parameters**:
```json
{
  "kind": {
    "type": "string",
    "description": "Controller kind (source-controller, kustomize-controller, helm-controller, notification-controller, image-reflector-controller, image-automation-controller)",
    "enum": ["source-controller", "kustomize-controller", "helm-controller", "notification-controller", "image-reflector-controller", "image-automation-controller"],
    "default": "kustomize-controller"
  },
  "namespace": {
    "type": "string",
    "description": "Flux namespace (default: flux-system)",
    "default": "flux-system"
  },
  "tail": {
    "type": "integer",
    "description": "Number of lines to show from the end",
    "default": 100
  },
  "since": {
    "type": "string",
    "description": "Show logs since duration (e.g., '5m', '1h')"
  }
}
```

**Output Format**: Controller logs.

**Example Response**:
```json
{
  "success": true,
  "data": {
    "controller": "kustomize-controller",
    "namespace": "flux-system",
    "logs": "2025-12-23T10:30:00.000Z info Kustomization/infrastructure.flux-system - Reconciliation finished in 2.5s\n..."
  }
}
```

## Configuration

### Tool Configuration

```yaml
tools:
  - type: flux
    config:
      kubeconfig: /home/user/.kube/config  # Optional: defaults to default kubeconfig
      context: production                   # Optional: kubectl context to use
      namespace: flux-system                # Default namespace for operations
```

### Prerequisites

1. **Flux CLI**: Must be installed and available in PATH
   ```bash
   curl -s https://fluxcd.io/install.sh | sudo bash
   ```

2. **kubectl**: Required for Kubernetes API access
   ```bash
   # Verify kubectl is configured
   kubectl cluster-info
   ```

3. **Flux Installation**: Flux controllers must be running in the cluster
   ```bash
   flux check
   ```

4. **RBAC Permissions**: Agent service account needs:
   - Read access to Kustomizations, HelmReleases, GitRepositories, HelmRepositories
   - Write access for reconcile, suspend, resume operations

### Environment Variables

- `KUBECONFIG`: Path to kubeconfig file (optional, defaults to `~/.kube/config`)
- `FLUX_NAMESPACE`: Default namespace for Flux operations (optional, defaults to `flux-system`)

## Implementation Details

### CLI Execution

The Flux Tool uses `tokio::process::Command` to execute `flux` CLI commands asynchronously.

**Pattern**:
```rust
use tokio::process::Command;

let mut cmd = Command::new("flux");
cmd.args(&["get", "kustomizations", "-o", "json"]);

if let Some(ns) = namespace {
    cmd.args(&["-n", ns]);
}

// Set kubeconfig if configured
if let Some(kubeconfig) = &self.config.kubeconfig {
    cmd.env("KUBECONFIG", kubeconfig);
}

let output = cmd.output().await?;
```

### Output Parsing

All Flux CLI commands use `-o json` flag for structured output:

```rust
// Parse JSON output
let stdout = String::from_utf8_lossy(&output.stdout);
let data: serde_json::Value = serde_json::from_str(&stdout)
    .map_err(|e| format!("Failed to parse Flux output: {}", e))?;
```

**Fallback Handling**:
- If JSON parsing fails, return raw stdout as text
- Include stderr in error responses
- Check exit code for command success

### Error Handling

**Common Error Scenarios**:

1. **Flux Not Installed**:
   ```json
   {
     "success": false,
     "error": "flux command not found. Install Flux CLI: curl -s https://fluxcd.io/install.sh | sudo bash"
   }
   ```

2. **Flux Not Initialized**:
   ```json
   {
     "success": false,
     "error": "Flux not installed in cluster. Run: flux install"
   }
   ```

3. **Resource Not Found**:
   ```json
   {
     "success": false,
     "error": "Kustomization 'my-app' not found in namespace 'default'"
   }
   ```

4. **Permission Denied**:
   ```json
   {
     "success": false,
     "error": "Insufficient permissions to reconcile kustomization/infrastructure"
   }
   ```

5. **Kubeconfig Invalid**:
   ```json
   {
     "success": false,
     "error": "Unable to connect to cluster. Check kubeconfig at /path/to/config"
   }
   ```

### Timeout Management

**Default Timeouts**:
- List operations: 30 seconds
- Get operations: 30 seconds
- Reconcile operations: 120 seconds (reconciliation can be slow)
- Logs operations: 30 seconds
- Suspend/Resume operations: 30 seconds

**Configurable Timeout**:
```yaml
tools:
  - type: flux
    config:
      timeout_secs: 120  # Override default timeout
```

## Tool Parameters Schema

### flux_kustomization_list

```json
{
  "type": "object",
  "properties": {
    "namespace": {
      "type": "string",
      "description": "Filter by namespace (optional)"
    },
    "all_namespaces": {
      "type": "boolean",
      "description": "List from all namespaces",
      "default": true
    }
  },
  "required": []
}
```

### flux_kustomization_get

```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "Kustomization name"
    },
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace",
      "default": "flux-system"
    }
  },
  "required": ["name"]
}
```

### flux_helmrelease_list

```json
{
  "type": "object",
  "properties": {
    "namespace": {
      "type": "string",
      "description": "Filter by namespace (optional)"
    },
    "all_namespaces": {
      "type": "boolean",
      "description": "List from all namespaces",
      "default": true
    }
  },
  "required": []
}
```

### flux_helmrelease_get

```json
{
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "description": "HelmRelease name"
    },
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace"
    }
  },
  "required": ["name", "namespace"]
}
```

### flux_reconcile

```json
{
  "type": "object",
  "properties": {
    "kind": {
      "type": "string",
      "description": "Resource kind",
      "enum": ["kustomization", "helmrelease", "source", "gitrepository", "helmrepository", "helmchart"]
    },
    "name": {
      "type": "string",
      "description": "Resource name"
    },
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace",
      "default": "flux-system"
    },
    "with_source": {
      "type": "boolean",
      "description": "Reconcile source first",
      "default": false
    }
  },
  "required": ["kind", "name"]
}
```

### flux_suspend

```json
{
  "type": "object",
  "properties": {
    "kind": {
      "type": "string",
      "description": "Resource kind",
      "enum": ["kustomization", "helmrelease"]
    },
    "name": {
      "type": "string",
      "description": "Resource name"
    },
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace",
      "default": "flux-system"
    }
  },
  "required": ["kind", "name"]
}
```

### flux_resume

```json
{
  "type": "object",
  "properties": {
    "kind": {
      "type": "string",
      "description": "Resource kind",
      "enum": ["kustomization", "helmrelease"]
    },
    "name": {
      "type": "string",
      "description": "Resource name"
    },
    "namespace": {
      "type": "string",
      "description": "Kubernetes namespace",
      "default": "flux-system"
    }
  },
  "required": ["kind", "name"]
}
```

### flux_logs

```json
{
  "type": "object",
  "properties": {
    "kind": {
      "type": "string",
      "description": "Controller kind",
      "enum": ["source-controller", "kustomize-controller", "helm-controller", "notification-controller", "image-reflector-controller", "image-automation-controller"],
      "default": "kustomize-controller"
    },
    "namespace": {
      "type": "string",
      "description": "Flux namespace",
      "default": "flux-system"
    },
    "tail": {
      "type": "integer",
      "description": "Number of lines from the end",
      "default": 100
    },
    "since": {
      "type": "string",
      "description": "Show logs since duration (e.g., '5m', '1h')"
    }
  },
  "required": []
}
```

## Example Usage in Agent YAML

### Basic GitOps Operations

```yaml
spec:
  name: gitops-operator
  description: Manages GitOps deployments via Flux

  model:
    provider: google
    name: gemini-2.5-flash

  tools:
    - flux_kustomization_list
    - flux_kustomization_get
    - flux_helmrelease_list
    - flux_helmrelease_get
    - flux_reconcile
    - flux_suspend
    - flux_resume
    - flux_logs

  config:
    flux:
      namespace: flux-system
      kubeconfig: /etc/kubeconfig/production.yaml
```

### Agent Workflow Example

```yaml
# Example: Deploy new application version
spec:
  name: deployment-agent
  system: |
    You are a deployment agent responsible for managing Flux-based GitOps deployments.

    When asked to deploy a new version:
    1. Check current Kustomization status
    2. Trigger reconciliation with source
    3. Monitor reconciliation progress
    4. Report success or failure

  tools:
    - flux_kustomization_get
    - flux_reconcile
    - flux_logs
    - kubectl  # For checking pod status
```

### RCA Investigation Example

```yaml
# Example: Investigate failed deployment
spec:
  name: rca-agent
  system: |
    You investigate deployment failures in Flux-managed clusters.

    Steps:
    1. List all Kustomizations and HelmReleases
    2. Identify resources in error state
    3. Get detailed status and conditions
    4. Retrieve controller logs for errors
    5. Check related Kubernetes resources
    6. Provide root cause analysis

  tools:
    - flux_kustomization_list
    - flux_helmrelease_list
    - flux_kustomization_get
    - flux_helmrelease_get
    - flux_logs
    - kubectl
```

### Emergency Response Example

```yaml
# Example: Suspend problematic deployment
spec:
  name: incident-response-agent
  system: |
    You respond to production incidents involving Flux deployments.

    Emergency actions:
    1. Suspend the failing resource
    2. Get current status
    3. Check logs for errors
    4. Coordinate with teams
    5. Resume after fix is confirmed

  tools:
    - flux_suspend
    - flux_kustomization_get
    - flux_helmrelease_get
    - flux_logs
    - flux_resume
```

## Security Considerations

### RBAC Requirements

**Minimum Required Permissions**:
```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: flux-agent-reader
rules:
  - apiGroups: ["kustomize.toolkit.fluxcd.io"]
    resources: ["kustomizations"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["helm.toolkit.fluxcd.io"]
    resources: ["helmreleases"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["source.toolkit.fluxcd.io"]
    resources: ["gitrepositories", "helmrepositories", "helmcharts"]
    verbs: ["get", "list", "watch"]
```

**Write Permissions** (for reconcile/suspend/resume):
```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: flux-agent-operator
rules:
  - apiGroups: ["kustomize.toolkit.fluxcd.io"]
    resources: ["kustomizations"]
    verbs: ["get", "list", "watch", "patch"]
  - apiGroups: ["helm.toolkit.fluxcd.io"]
    resources: ["helmreleases"]
    verbs: ["get", "list", "watch", "patch"]
```

### Sensitive Data Handling

1. **Kubeconfig Protection**: Never log or expose kubeconfig contents
2. **Namespace Isolation**: Respect namespace boundaries unless explicitly granted cluster-wide access
3. **Audit Logging**: Log all reconcile/suspend/resume operations
4. **Rate Limiting**: Implement backoff for failed reconciliations

### Multi-tenancy

For multi-tenant environments:
```yaml
# Namespace-scoped configuration
spec:
  config:
    flux:
      namespace: team-a  # Restrict to specific namespace
      enforce_namespace: true  # Prevent cross-namespace operations
```

## Performance Considerations

### Caching Strategy

- Cache Kustomization/HelmRelease list for 30 seconds
- Invalidate cache on reconcile/suspend/resume operations
- Use `kubectl get` directly for critical status checks

### Parallel Operations

Execute multiple status checks in parallel:
```rust
// Pseudo-code
let (kustomizations, helmreleases) = tokio::join!(
    flux_kustomization_list(input),
    flux_helmrelease_list(input)
);
```

### Resource Limits

- Limit log tail to 1000 lines maximum
- Implement pagination for large resource lists
- Use field selectors to reduce API load

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flux_availability() {
        assert!(FluxTool::is_available());
    }

    #[test]
    fn test_kustomization_list_schema() {
        let tool = FluxKustomizationListTool::new();
        let schema = &tool.config().parameters;
        assert!(schema.get("properties").is_some());
    }
}
```

### Integration Tests

```bash
# Verify Flux CLI integration
cargo test --test flux_integration -- --nocapture

# Test scenarios:
# 1. List kustomizations in test cluster
# 2. Get specific kustomization
# 3. Reconcile test resource
# 4. Suspend and resume
```

## Monitoring and Observability

### Metrics to Track

- Reconciliation trigger count
- Reconciliation success/failure rate
- Average reconciliation duration
- Suspended resource count
- Controller log error rate

### Health Checks

```yaml
spec:
  health_check:
    command: flux check
    interval: 60s
    timeout: 10s
```

## Migration Path

For clusters not using Flux:

1. **Detection**: Check if Flux is installed via `flux check`
2. **Graceful Degradation**: Fall back to kubectl tools if Flux unavailable
3. **Installation Guidance**: Provide instructions for Flux installation
4. **Compatibility**: Support both Flux v1 (legacy) and v2 (current)

## Future Enhancements

### Planned Features

1. **Image Automation**: Support for Flux image update automation
2. **Source Operations**: Manage GitRepositories and HelmRepositories
3. **Notification**: Integration with Flux notification controller
4. **Bootstrap**: Support `flux bootstrap` for cluster initialization
5. **Export/Import**: Export Flux resources for disaster recovery

### API Evolution

Consider adding:
- `flux_export`: Export resource configurations
- `flux_stats`: Get reconciliation statistics
- `flux_tree`: Show resource dependency tree
- `flux_events`: Get Kubernetes events for Flux resources

## References

- [Flux Documentation](https://fluxcd.io/flux/)
- [Flux CLI Reference](https://fluxcd.io/flux/cmd/)
- [Flux API Reference](https://fluxcd.io/flux/components/)
- [GitOps Toolkit](https://fluxcd.io/flux/components/)
- [Flux Best Practices](https://fluxcd.io/flux/guides/)

## Appendix

### Flux Resource Types

| Resource | API Group | Purpose |
|----------|-----------|---------|
| Kustomization | kustomize.toolkit.fluxcd.io | Apply Kustomize overlays from Git |
| HelmRelease | helm.toolkit.fluxcd.io | Deploy Helm charts |
| GitRepository | source.toolkit.fluxcd.io | Git source for manifests |
| HelmRepository | source.toolkit.fluxcd.io | Helm chart repository |
| HelmChart | source.toolkit.fluxcd.io | Helm chart source |
| ImageRepository | image.toolkit.fluxcd.io | Container image repository |
| ImagePolicy | image.toolkit.fluxcd.io | Image tag filtering policy |
| ImageUpdateAutomation | image.toolkit.fluxcd.io | Automated image updates |

### Common Flux Commands

| Command | Purpose |
|---------|---------|
| `flux get all` | List all Flux resources |
| `flux check` | Verify Flux installation |
| `flux reconcile source git <name>` | Sync Git source |
| `flux create source git <name>` | Create GitRepository |
| `flux export kustomization <name>` | Export resource YAML |
| `flux trace <name>` | Show resource lineage |
