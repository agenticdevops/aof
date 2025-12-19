# Platform Policies

AOF uses simple platform-based safety rules to protect against accidental destructive commands.

## Default Policies

| Platform | Read | Write | Why |
|----------|------|-------|-----|
| CLI | Yes | Yes | Local, authenticated |
| Slack | Yes | Yes (with approval) | Enterprise SSO, audit logs |
| Telegram | Yes | No | Mobile, less controlled |
| WhatsApp | Yes | No | Mobile, less controlled |

## How It Works

1. Message arrives from a platform (Telegram, Slack, etc.)
2. AOF checks if the command is a write operation
3. If write + mobile platform → Block with helpful message
4. If write + Slack → Execute (approval workflow available)
5. If read → Execute on all platforms

## Write Operations

These patterns are blocked on mobile platforms:

**kubectl:**
- `kubectl apply`, `kubectl create`, `kubectl delete`
- `kubectl patch`, `kubectl replace`, `kubectl edit`
- `kubectl scale`, `kubectl rollout`, `kubectl drain`

**docker:**
- `docker rm`, `docker rmi`, `docker stop`, `docker kill`
- `docker run`, `docker exec`, `docker build`

**helm:**
- `helm install`, `helm upgrade`, `helm uninstall`, `helm rollback`

**terraform:**
- `terraform apply`, `terraform destroy`, `terraform import`

**aws:**
- Commands with `create`, `delete`, `update`, `terminate`, `modify`

**git:**
- `git push`, `git reset --hard`, `git rebase`

## Blocked Message

When a write operation is blocked:

```
Write operations are blocked on Telegram.
Use Slack or CLI for this operation.
```

## Slack Approval Workflow

On Slack, destructive commands can require approval:

1. User requests: "delete the nginx pod"
2. Bot asks for approval with ✅/❌ buttons
3. Authorized user approves
4. Command executes

Configure approvers in your config:

```yaml
spec:
  platforms:
    slack:
      approval_allowed_users:
        - U015VBH1GTZ  # Slack user ID
```

## Related

- [Safety Layer Guide](../guides/safety-layer.md)
- [Approval Workflow](../guides/approval-workflow.md)
