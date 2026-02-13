# Sandbox Isolation Architecture

## Overview

Sandbox isolation executes untrusted or high-risk tools in Docker containers with defense-in-depth security restrictions. This prevents malicious or buggy tools from escaping the container, accessing credentials, or impacting the host system.

## Problem Statement

Autonomous agents execute tools provided by operators or discovered from external sources. These tools may be:
- **Untrusted:** From third-party skill libraries or user-supplied
- **Buggy:** Tools with command injection vulnerabilities
- **Malicious:** Intentionally designed to escape sandbox

Without isolation, a compromised tool could:
- Access all agent credentials
- Escape to host system via privilege escalation
- Impact other agents or host services
- Exfiltrate sensitive data

Sandboxing ensures even if tool is compromised, damage is limited to the container.

## Architecture

### Defense-in-Depth Layers

1. **User Namespaces:** Container runs as unprivileged user (1000:1000)
2. **Read-only Root Filesystem:** Tool cannot modify system files
3. **Resource Limits:** Memory (512MB), CPU (1 core), PIDs (100)
4. **Seccomp Profile:** Restricts dangerous syscalls
5. **No Network Access:** Tools cannot reach external systems (unless explicitly enabled)
6. **Credential Isolation:** Credentials mounted read-only via file permissions (0400)

### Sandbox Configuration

```rust
pub struct SandboxConfig {
    pub image: String,                    // "aof-sandbox:latest"
    pub memory_mb: u64,                   // 512 MB limit
    pub cpu_limit: f64,                   // 1.0 CPU
    pub pids_limit: i64,                  // 100 max processes
    pub read_only_root: bool,             // true
    pub tmpfs_size_mb: u64,               // 100 MB for /tmp
    pub user: String,                     // "1000:1000" (unprivileged)
    pub seccomp_profile: Option<PathBuf>, // Path to seccomp JSON
}
```

### Seccomp Profile

Seccomp filters syscalls to prevent privilege escalation and dangerous operations:

**Allowed syscalls:**
- Read, write, open, close (I/O)
- Socket, connect, listen (networking)
- Clone, fork, execve (process management)
- Chmod, chown (permission changes within container)

**Blocked syscalls:**
- `ptrace` — Prevent debugging/introspection
- `setuid`, `setgid` — Prevent privilege escalation
- `mount`, `umount` — Prevent filesystem modifications
- `init_module`, `delete_module` — Prevent kernel modules
- Raw sockets — Prevent network sniffing

See `configs/seccomp-profile.json` for complete list.

## Risk-Based Sandboxing

Not all tools need containerization. RiskPolicy evaluates context and determines execution mode:

### Decision Matrix

| Environment | Operation Type | Decision | Reason |
|-------------|-----------------|----------|--------|
| Dev | Read-only | Sandbox | Always protect in dev |
| Dev | Write | Sandbox | Always protect in dev |
| Dev | Destructive | Sandbox | Always protect in dev |
| Prod | Read-only | Host Trusted | Fast path for safe ops |
| Prod | Write | Sandbox | Protect from bugs |
| Prod | Destructive | Sandbox | High risk, always isolate |

### Operation Classification

```rust
fn is_destructive(&self, tool: &str, args: &[String]) -> bool {
    // Examples: delete, remove, rm, kill, stop, restart, scale, terminate
}

fn is_write(&self, tool: &str, args: &[String]) -> bool {
    // Examples: apply, patch, create, set, update, edit
}

// Everything else is read-only (get, describe, logs, query)
```

## Integration

### ToolExecutor Integration

ToolExecutor evaluates risk and decides execution mode:

```rust
pub async fn execute(&self, tool_name: &str, input: &ToolInput) -> Result<ToolResult> {
    // 1. Evaluate risk
    let decision = self.risk_policy.should_sandbox(&context, tool, args);

    // 2. Execute accordingly
    match decision {
        SandboxingDecision::Sandbox => {
            // Run in Docker container
            self.sandbox.execute(tool, args, options).await?
        }
        SandboxingDecision::HostWithRestrictions => {
            // Run on host with seccomp
            tokio::process::Command::new(tool).args(args).output().await?
        }
        SandboxingDecision::HostTrusted => {
            // Run on host without restrictions (fast path)
            tokio::process::Command::new(tool).args(args).output().await?
        }
    }
}
```

### Credential Access Control

Sensitive credentials mounted read-only into sandbox:

```rust
ContainerOptions {
    mounts: vec![
        ("/var/aof/creds/agent-001/k8s", "/creds/k8s", "ro"),
        ("/var/aof/creds/agent-001/aws", "/creds/aws", "ro"),
    ],
    ...
}
```

File permissions prevent modification:
```bash
# Credentials owned by system, readable by unprivileged user (0400)
-r--------  1 root root 2048 Feb 13 10:00 /var/aof/creds/agent-001/k8s
```

Tool can **read** credentials but cannot **modify** or **write** files.

## Configuration

### Environment Variables

```bash
# Sandbox image
export AOF_SANDBOX_IMAGE=aof-sandbox:latest

# Resource limits
export AOF_SANDBOX_MEMORY_MB=512
export AOF_SANDBOX_CPU_LIMIT=1.0
export AOF_SANDBOX_PIDS_LIMIT=100

# Seccomp profile
export AOF_SECCOMP_PROFILE=/etc/aof/seccomp-profile.json

# Disable sandboxing (not recommended)
export AOF_SANDBOXING_ENABLED=false
```

### CLI Flags

```bash
aofctl serve \
  --sandbox-image aof-sandbox:latest \
  --sandbox-memory 512 \
  --disable-sandbox  # (for testing only)
```

### YAML Configuration

```yaml
apiVersion: aof.dev/v1
kind: ServeConfig
metadata:
  name: default
spec:
  sandbox:
    enabled: true
    image: aof-sandbox:latest
    memory_mb: 512
    cpu_limit: 1.0
    pids_limit: 100
    seccomp_profile: /etc/aof/seccomp-profile.json

  risk_policy:
    enabled: true
    default_sandbox_on_dev: true
    default_sandbox_on_prod_destructive: true
```

## Operation

### Sandbox Lifecycle

1. **Create:** Docker creates container with resource limits
2. **Start:** Container starts, executes tool command
3. **Monitor:** System polls container status every 100ms
4. **Timeout:** If running >60s, container is killed
5. **Logs:** Tool output captured from container logs
6. **Cleanup:** Container removed (prevents garbage accumulation)

### Tool Execution

```bash
# Inside sandbox
docker run --rm \
  --user 1000:1000 \
  --memory 512m \
  --cpus 1.0 \
  --pids-limit 100 \
  --read-only \
  --security-opt seccomp=/etc/aof/seccomp-profile.json \
  --mount type=tmpfs,destination=/tmp,tmpfs-size=100m \
  -v /var/aof/creds/agent-001:/creds:ro \
  aof-sandbox:latest \
  kubectl get pods
```

### Example: Kubectl Delete

```rust
// Agent executes kubectl delete
tool_executor.execute("kubectl", &["delete", "pod", "api-001"]).await?

// Evaluation:
// 1. is_destructive("kubectl", ["delete", ...]) → true
// 2. context = Production
// 3. decision = Sandbox (destructive in prod)

// Execution:
// 1. Acquire resource lock for "pod:prod/api-001"
// 2. Create Docker container
// 3. Mount credentials read-only
// 4. Execute: kubectl delete pod api-001
// 5. Wait for completion
// 6. Capture output
// 7. Remove container
// 8. Release lock
// 9. Log decision with outcome
```

## Monitoring

### Decision Log

Each sandbox execution logged:

```json
{
  "agent_id": "incident-handler-001",
  "action": "sandbox_execute",
  "tool": "kubectl",
  "operation": "delete pod",
  "timestamp": "2026-02-13T10:23:45.123Z",
  "confidence": 0.95,
  "metadata": {
    "decision": "Sandbox",
    "memory_mb": 512,
    "cpu_limit": 1.0,
    "timeout_seconds": 60,
    "output_length": 245
  }
}
```

### Querying Sandbox Execution

```bash
# Find all sandboxed operations
aof query "action=sandbox_execute"

# Find sandbox timeouts
aof query "action=sandbox_execute AND metadata.timeout_reached=true"

# Find credential mount failures
aof query "action=credential_mount_error"

# Find operations by tool type
aof query "action=sandbox_execute AND tool=kubectl"
```

## Troubleshooting

### Docker Daemon Not Accessible

**Symptom:** `Docker daemon not accessible: Cannot connect to docker.sock`

**Causes:**
1. Docker daemon not running
2. Socket permission issue
3. Running in non-Linux environment

**Solutions:**
```bash
# Verify daemon running
docker ps

# Fix socket permissions (if needed)
sudo chmod 666 /var/run/docker.sock

# Fallback to host execution (not recommended)
aofctl serve --disable-sandbox
```

### Sandbox Timeout

**Symptom:** `Sandbox execution timeout: tool execution took >60 seconds`

**Causes:**
1. Tool performing long-running operation
2. Container resource limits too restrictive
3. Network issues (if container has network access)

**Solutions:**
- Increase timeout: `--sandbox-timeout 120`
- Increase memory: `--sandbox-memory 1024`
- Check tool logs for bottlenecks

### Permission Denied

**Symptom:** `Permission denied` when executing tool in sandbox

**Causes:**
1. Tool requires root (but container runs as 1000:1000)
2. Credential file not readable by unprivileged user
3. Tool trying to write to read-only filesystem

**Solutions:**
```bash
# Verify credential permissions
ls -la /var/aof/creds/agent-001/k8s
# Should be -r--------  1 root root ...

# Enable write access to /tmp (already enabled via tmpfs)
# For other write locations, use tmpfs mounts

# If tool requires root, configure via YAML:
# Note: This bypasses security restrictions — use carefully
```

### Seccomp Violation

**Symptom:** `Operation not permitted` inside sandbox

**Cause:** Seccomp profile blocks syscall used by tool

**Solutions:**
1. Update tool to use allowed syscalls (preferred)
2. Extend seccomp profile (less secure)
3. Use HostWithRestrictions mode (medium security)

Check which syscall failed:
```bash
# Enable seccomp logging (requires kernel support)
docker logs <container> 2>&1 | grep SCMP
```

## Performance

### Latency Impact

- **Container creation:** 200-500ms
- **Tool execution:** Depends on tool
- **Log capture:** 50-100ms
- **Container cleanup:** 100-200ms
- **Total overhead:** 350-800ms per execution

For read-only operations in prod (HostTrusted path): 0ms overhead

### Resource Consumption

Per execution:
- **Memory:** 512MB (temporary, released after execution)
- **CPU:** Capped at 1 core
- **Disk:** Cleanup removes container (no accumulation)
- **Network:** None (unless explicitly enabled)

Concurrent executions on 4-core system:
- 4 tools running in parallel: Each gets 1 CPU max, 512MB mem per tool
- No impact to host or other agents

## Security Guarantees

### What Sandbox Prevents

✓ Privilege escalation (no setuid/capset)
✓ Filesystem escape (read-only root)
✓ Kernel manipulation (no module loading)
✓ Credential exfiltration (file permissions enforce read-only)
✓ Network escape (no network access by default)
✓ Process explosion (PID limit)
✓ Memory exhaustion (512MB limit)

### What Sandbox Does NOT Prevent

✗ Logic bugs in tools (incorrect operations still execute)
✗ Unauthorized tool execution (relies on tool discovery controls)
✗ Data destruction within sandbox scope (authorized operations)

## Future Enhancements

### Phase 3: Enhanced Isolation
- gVisor integration (stronger isolation than seccomp alone)
- Device pairing (advanced resource constraints)
- Credential rotation on tool compromise detection

### Phase 8: Production Hardening
- Custom sandbox images per skill type
- Adaptive resource limits based on tool requirements
- Sandbox failure autopsy (post-mortem analysis of crashes)

## See Also

- [Seccomp Profile](/configs/seccomp-profile.json)
- [Risk Policy](/docs/dev/resource-locking.md#risk-based-sandboxing)
- [ToolExecutor Integration](/docs/dev/tool-executor.md)
