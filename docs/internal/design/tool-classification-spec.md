# Internal Design: Tool Classification Specification

**Status**: Design Approved
**Author**: AOF Team
**Created**: December 19, 2025
**Parent**: [Telegram Mobile Companion](./telegram-mobile-companion.md)

---

## Overview

This document specifies the Tool Classification system that enables platform-aware safety controls across all AOF agents. The system classifies tool commands into verb classes (read, write, delete, dangerous) and enforces policies based on platform and context.

## Design Goals

1. **Extensible**: Add new tools without code changes
2. **Fallback**: Unknown tools work via pattern matching
3. **Configurable**: Enterprises can override defaults
4. **Performance**: Fast classification with compiled regex

## Resource Schema

### ToolClassification

```yaml
apiVersion: aof.dev/v1
kind: ToolClassification
metadata:
  name: default-classifications
  labels:
    scope: builtin
spec:
  tools:
    <tool_name>:
      read: [<verbs>]
      write: [<verbs>]
      delete: [<verbs>]
      dangerous: [<verbs>]

  generic_patterns:
    read: [<regex_patterns>]
    write: [<regex_patterns>]
    delete: [<regex_patterns>]
    dangerous: [<regex_patterns>]
```

## Built-in Tool Classifications

### Kubernetes (kubectl)

```yaml
kubectl:
  read:
    - get
    - list
    - describe
    - logs
    - top
    - explain
    - auth
    - api-resources
    - api-versions
    - config view
    - config get-contexts
    - cluster-info
    - version
  write:
    - apply
    - create
    - patch
    - scale
    - set
    - label
    - annotate
    - taint
    - cordon
    - uncordon
    - drain
    - rollout
    - edit
    - replace
    - autoscale
    - expose
  delete:
    - delete
  dangerous:
    - exec
    - cp
    - port-forward
    - attach
    - debug
    - run --rm  # Interactive pods
```

**Rationale**:
- `exec`/`cp` classified as dangerous because they can exfiltrate data
- `port-forward` can expose internal services
- `drain` is write (not delete) because nodes can be uncordoned

### Docker

```yaml
docker:
  read:
    - ps
    - images
    - logs
    - inspect
    - info
    - version
    - stats
    - top
    - history
    - diff
    - events
    - port
    - network ls
    - volume ls
    - system df
  write:
    - run
    - build
    - push
    - pull
    - tag
    - start
    - restart
    - pause
    - unpause
    - commit
    - import
    - load
    - save
    - network create
    - volume create
  delete:
    - rm
    - rmi
    - stop
    - kill
    - prune
    - network rm
    - volume rm
    - system prune
  dangerous:
    - exec
    - attach
    - --privileged
```

### Helm

```yaml
helm:
  read:
    - list
    - status
    - get
    - show
    - history
    - search
    - template
    - verify
    - repo list
    - env
    - version
  write:
    - install
    - upgrade
    - rollback
    - repo add
    - repo update
    - plugin install
    - pull
    - push
    - package
  delete:
    - uninstall
    - repo remove
    - plugin uninstall
  dangerous: []
```

### Terraform

```yaml
terraform:
  read:
    - show
    - state list
    - state show
    - plan
    - output
    - providers
    - validate
    - version
    - fmt -check
    - graph
    - workspace list
  write:
    - apply
    - import
    - taint
    - untaint
    - refresh
    - init
    - workspace new
    - workspace select
    - fmt
  delete:
    - destroy
    - state rm
    - workspace delete
  dangerous:
    - apply -auto-approve
    - destroy -auto-approve
```

### Git

```yaml
git:
  read:
    - status
    - log
    - diff
    - show
    - branch
    - tag
    - remote
    - fetch
    - ls-files
    - ls-remote
    - blame
    - shortlog
    - describe
    - rev-parse
    - config --list
  write:
    - add
    - commit
    - push
    - pull
    - merge
    - rebase
    - checkout
    - switch
    - stash
    - cherry-pick
    - am
    - format-patch
    - tag -a
    - branch -m
    - config --global
  delete:
    - reset --hard
    - clean -fd
    - branch -D
    - push --delete
    - tag -d
    - stash drop
    - stash clear
  dangerous:
    - push --force
    - push -f
    - rebase -i
    - filter-branch
    - gc --prune
```

### AWS CLI

```yaml
aws:
  read:
    - describe
    - list
    - get
    - s3 ls
    - sts get-caller-identity
    - iam list-users
    - ec2 describe-instances
    - rds describe-db-instances
    - eks describe-cluster
    - logs get-log-events
    - cloudwatch get-metric-data
  write:
    - create
    - put
    - update
    - start
    - run
    - s3 cp
    - s3 sync
    - ec2 run-instances
    - ec2 start-instances
    - ec2 stop-instances
    - rds start-db-instance
    - rds stop-db-instance
    - eks update-cluster
    - lambda update-function
  delete:
    - delete
    - terminate
    - s3 rm
    - ec2 terminate-instances
    - rds delete-db-instance
    - eks delete-cluster
    - lambda delete-function
  dangerous:
    - iam create-user
    - iam create-access-key
    - iam attach-policy
    - s3 rm --recursive
    - s3api delete-bucket
```

### Google Cloud (gcloud)

```yaml
gcloud:
  read:
    - describe
    - list
    - get
    - info
    - config list
    - auth list
    - compute instances list
    - container clusters list
    - sql instances list
  write:
    - create
    - update
    - set
    - deploy
    - compute instances create
    - compute instances start
    - compute instances stop
    - container clusters create
    - container clusters resize
    - sql instances patch
  delete:
    - delete
    - compute instances delete
    - container clusters delete
    - sql instances delete
  dangerous:
    - iam service-accounts create
    - iam service-accounts keys create
```

### Prometheus (promtool)

```yaml
promtool:
  read:
    - query
    - query instant
    - query range
    - check config
    - check rules
    - check metrics
    - tsdb list
  write:
    - push metrics
  delete: []
  dangerous: []
```

### systemctl

```yaml
systemctl:
  read:
    - status
    - list-units
    - list-unit-files
    - is-active
    - is-enabled
    - show
    - cat
  write:
    - start
    - stop
    - restart
    - reload
    - enable
    - disable
  delete:
    - mask
    - unmask
  dangerous:
    - daemon-reload
    - daemon-reexec
```

## Generic Pattern Fallback

For tools not explicitly defined, these regex patterns classify commands:

```yaml
generic_patterns:
  read:
    # Command starts with read verbs
    - "^(get|list|show|describe|status|info|inspect|cat|head|tail|watch|ls|ps|version|check|validate)\\b"
    # Common read flags
    - "\\b(--help|-h|--version|--dry-run)$"
    # Query-like patterns
    - "\\bquery\\b"
    - "\\bfetch\\b"
    - "\\bread\\b"

  write:
    # Command starts with write verbs
    - "^(create|apply|patch|update|set|add|push|install|upgrade|run|start|exec|deploy|scale|restart|enable|configure)\\b"
    # Common write patterns
    - "\\bwrite\\b"
    - "\\bsync\\b"
    - "\\bimport\\b"

  delete:
    # Command starts with delete verbs
    - "^(delete|destroy|remove|rm|uninstall|terminate|kill|stop|drop|purge|prune|disable)\\b"
    # Destructive patterns
    - "\\btruncate\\b"
    - "\\bclear\\b"
    - "\\bwipe\\b"

  dangerous:
    # Force flags
    - "\\b(--force|-f)\\b"
    # Recursive operations
    - "\\b(--all|--recursive|-r|-R)\\b"
    # Privilege escalation
    - "\\bsudo\\b"
    # Classic dangerous patterns
    - "\\brm\\s+-rf\\b"
    - "\\brm\\s+-r\\b"
    # Shell redirects (potential data exfil)
    - "\\b>\\b"
    - "\\b>>\\b"
    - "\\b\\|\\b"  # Pipes
    # Credential operations
    - "\\b(password|secret|token|credential)\\b"
```

## Classification Priority

Commands are classified in this order (first match wins):

1. **Dangerous patterns** (highest priority)
2. **Tool-specific verbs**
3. **Generic patterns**
4. **Default**: Assume `write` (fail secure)

```rust
fn classify(tool: &str, command: &str) -> VerbClass {
    // 1. Check dangerous patterns first (override everything)
    if matches_dangerous_patterns(command) {
        return VerbClass::Dangerous;
    }

    // 2. Check tool-specific classification
    if let Some(tool_class) = classify_by_tool(tool, command) {
        return tool_class;
    }

    // 3. Check generic patterns
    if let Some(generic_class) = classify_by_patterns(command) {
        return generic_class;
    }

    // 4. Default to write (safer than read for unknown)
    VerbClass::Write
}
```

## Example Classifications

| Command | Tool | Verb | Class | Reason |
|---------|------|------|-------|--------|
| `kubectl get pods` | kubectl | get | read | Tool-specific |
| `kubectl delete pod x` | kubectl | delete | delete | Tool-specific |
| `kubectl exec -it x -- sh` | kubectl | exec | dangerous | Tool-specific (data exfil risk) |
| `helm install x` | helm | install | write | Tool-specific |
| `docker rm -f x` | docker | rm | dangerous | Generic pattern (--force) |
| `mytool describe x` | mytool | describe | read | Generic pattern |
| `random delete-all` | random | delete | delete | Generic pattern |
| `unknown xyz` | unknown | xyz | write | Default (fail secure) |

## Configuration Examples

### Production Context (Strict)

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: prod
spec:
  namespace: production
  platform_policies:
    telegram:
      blocked_classes: [delete, dangerous]
      approval_classes: [write]
      allowed_classes: [read]
      blocked_message: |
        ⛔ This operation is blocked on Telegram for production.
        Use Slack or kubectl directly for write/delete operations.

    slack:
      blocked_classes: [dangerous]
      approval_classes: [delete, write]
      allowed_classes: [read]

    whatsapp:
      blocked_classes: [delete, dangerous, write]
      approval_classes: []
      allowed_classes: [read]
      blocked_message: "WhatsApp is read-only. Use Slack for changes."
```

### Development Context (Permissive)

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: dev
spec:
  namespace: development
  platform_policies:
    telegram:
      blocked_classes: [dangerous]
      approval_classes: []  # No approval in dev
      allowed_classes: [read, write, delete]

    slack:
      blocked_classes: []
      approval_classes: []
      allowed_classes: [read, write, delete, dangerous]
```

### Personal Cluster (Full Access)

```yaml
apiVersion: aof.dev/v1
kind: Context
metadata:
  name: personal
spec:
  namespace: default
  platform_policies:
    telegram:
      blocked_classes: []
      approval_classes: []
      allowed_classes: [read, write, delete, dangerous]
```

## Extension Points

### Adding a New Tool

1. Add to `examples/tool-classifications/default.yaml`:

```yaml
spec:
  tools:
    newtool:
      read: [status, list, show]
      write: [create, update, deploy]
      delete: [remove, destroy]
      dangerous: [--force-all]
```

2. Restart the AOF daemon (hot reload in future)

### Custom Enterprise Overrides

Create `tool-classifications/enterprise.yaml`:

```yaml
apiVersion: aof.dev/v1
kind: ToolClassification
metadata:
  name: acme-overrides
  labels:
    scope: enterprise
spec:
  tools:
    # Override kubectl to block exec entirely
    kubectl:
      read: [get, list, describe, logs]
      write: [apply, scale]
      delete: [delete]
      dangerous: [exec, cp, port-forward, apply, create, patch]

    # Add internal tools
    acme-deploy:
      read: [status, list]
      write: [deploy, rollback]
      delete: []
      dangerous: [--skip-validation]
```

## Performance Considerations

1. **Compiled Regex**: Generic patterns are compiled once at startup
2. **Tool Lookup**: O(1) HashMap lookup for tool-specific rules
3. **Short-circuit**: Dangerous patterns checked first (most restrictive)
4. **Caching**: Classification results can be cached by command hash

## Testing Requirements

### Unit Tests

- Each tool's complete verb list
- Generic pattern matching
- Priority/override behavior
- Edge cases (empty commands, unknown tools)

### Integration Tests

- End-to-end: Command → Classification → Policy → Result
- Cross-context testing (prod vs dev)
- Cross-platform testing (Telegram vs Slack)

---

## References

- [kubectl Command Reference](https://kubernetes.io/docs/reference/generated/kubectl/kubectl-commands)
- [Docker CLI Reference](https://docs.docker.com/engine/reference/commandline/cli/)
- [Helm Commands](https://helm.sh/docs/helm/)
- [Terraform Commands](https://developer.hashicorp.com/terraform/cli/commands)
- [AWS CLI Reference](https://awscli.amazonaws.com/v2/documentation/api/latest/reference/index.html)
- [gcloud Reference](https://cloud.google.com/sdk/gcloud/reference)
