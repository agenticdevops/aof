# Security Hardening - Developer Documentation

**Internal reference for AOF security architecture and implementation.**

## Overview

AOF's security hardening implements defense-in-depth for executing untrusted agent code against production infrastructure. This document describes the threat model, security layers, implementation details, and how to extend the security system.

## Threat Model

### Primary Threats

1. **Container Escape** (SEC-01)
   - **Attack**: Malicious agent exploits kernel vulnerability to escape sandbox
   - **Impact**: Full host compromise, access to all containers, credential exfiltration
   - **Mitigation**: Seccomp profiles, capability dropping, read-only root filesystem

2. **Credential Exfiltration** (SEC-02)
   - **Attack**: Compromised agent silently copies credentials (kubeconfig, AWS keys)
   - **Impact**: Unauthorized access to production clusters, data breaches
   - **Mitigation**: Credential access auditing, anomaly detection, behavioral baselines

3. **Resource Exhaustion** (SEC-03)
   - **Attack**: Agent consumes excessive CPU/memory/PIDs
   - **Impact**: Denial of service for other agents
   - **Mitigation**: Docker resource limits (already implemented in SandboxConfig)

4. **Privilege Escalation** (SEC-04)
   - **Attack**: Agent attempts to gain root or additional capabilities
   - **Impact**: Sandbox bypass, host compromise
   - **Mitigation**: --cap-drop=ALL, non-root user (UID 1000), seccomp blocking setuid/setgid

### Attack Vectors Blocked

- **Kernel exploits**: Seccomp blocks syscalls used in container escapes (ptrace, mount, bpf, io_uring)
- **Namespace manipulation**: setns, unshare blocked (prevents joining host namespaces)
- **Module loading**: init_module, finit_module blocked (prevents kernel module attacks)
- **eBPF attacks**: bpf syscall blocked
- **Capability acquisition**: All capabilities dropped by default

## Security Architecture

### Layer 1: Seccomp Profiles

**Location**: `crates/aof-runtime/src/sandbox/seccomp.rs`

**Purpose**: Restrict syscall surface area beyond Docker's default profile.

**Per-Tool Profiles**:

| Profile | File | Tools | Key Restrictions |
|---------|------|-------|------------------|
| default | `config/seccomp/default.json` | All unknown tools | Blocks 23 dangerous syscalls |
| kubectl | `config/seccomp/kubectl-profile.json` | kubectl, k9s | Same as default (uses kubeconfig file) |
| docker | `config/seccomp/docker-profile.json` | docker | Same as default (uses Docker socket) |
| readonly | `config/seccomp/readonly-profile.json` | cat, grep, ls | Only ~15 syscalls (read, stat, open with O_RDONLY) |

**Adding a New Profile**:

1. Create JSON profile in `config/seccomp/<name>-profile.json`:
   ```json
   {
     "defaultAction": "SCMP_ACT_ERRNO",
     "architectures": ["SCMP_ARCH_X86_64", "SCMP_ARCH_AARCH64"],
     "syscalls": [
       {
         "names": ["read", "write", "open"],
         "action": "SCMP_ACT_ALLOW"
       },
       {
         "names": ["ptrace", "mount"],
         "action": "SCMP_ACT_ERRNO"
       }
     ]
   }
   ```

2. Register in `SeccompProfileManager::new()`:
   ```rust
   SeccompProfile::new(
       "myprofile",
       profiles_dir.join("myprofile-profile.json"),
       vec!["mytool".to_string()],
   ),
   ```

3. Add test in `tests/sandbox_escape.rs`:
   ```rust
   #[test]
   fn test_myprofile_blocks_dangerous_syscalls() {
       // Verify critical syscalls are blocked
   }
   ```

**Validation**: Run `cargo test --test sandbox_escape` to ensure all critical syscalls are blocked.

### Layer 2: Capability Dropping

**Location**: `crates/aof-runtime/src/sandbox/capabilities.rs`

**Purpose**: Remove Linux capabilities, preventing privilege escalation.

**Default**: `--cap-drop=ALL` (no capabilities granted)

**Allowlist Exceptions**:

| Tool | Capability | Reason |
|------|-----------|--------|
| nc, socat, ncat | CAP_NET_BIND_SERVICE | Bind to ports below 1024 |

**Adding Capability Exceptions**:

1. Update `CapabilityConfig::for_tool()`:
   ```rust
   "mytool" => Self::new(
       true,
       vec!["CAP_SYS_ADMIN".to_string()],
   ),
   ```

2. **Justification required**: Document why the capability is needed in code comments.

3. **Security review**: Additional capabilities require approval (use GitHub issue/PR).

**Integration**: `Sandbox::security_args()` applies capability configuration automatically when constructing Docker containers.

### Layer 3: Credential Access Auditing

**Location**: `crates/aof-runtime/src/credential_audit.rs`

**Purpose**: Monitor credential access, detect exfiltration attempts.

**Components**:

1. **CredentialAccessInterceptor**:
   - Detects which credentials a tool will access (kubectl → Kubernetes, aws → AWS)
   - Logs every access with tamper-proof sequence numbers
   - Queries audit log for forensic analysis

2. **Audit Log Format** (`$DAEMON_DIR/credential-audit.log`):
   ```json
   {"event_id":"evt-1234-56","timestamp":"2026-02-14T12:30:00Z","agent_id":"agent-1","credential_type":"Kubernetes","file_path":"/home/.kube/config","access_mode":"Read","tool_context":{"tool_name":"kubectl","operation":"get pods","arguments":["get","pods"],"risk_level":"Low"},"anomaly_score":0.15,"sequence_number":42,"session_id":"session-1"}
   ```

3. **Tamper Detection**:
   - Sequence numbers are monotonically increasing
   - Gaps indicate log tampering (missing events)
   - Alerts generated via tracing::warn

**Interceptor Integration**:

```rust
let detector = Arc::new(AnomalyDetector::new());
let interceptor = CredentialAccessInterceptor::new(
    audit_log_path,
    detector,
);

// Before tool execution
let cred_types = interceptor.detect_credential_requirements(tool_name, &args);
for cred_type in cred_types {
    let anomaly = interceptor.check_access(&agent_id, &cred_type).await;
    if anomaly.recommended_action == AnomalyAction::Block {
        return Err(AofError::security("Credential access blocked"));
    }
}

// After tool execution
let event = interceptor.create_event(...);
interceptor.log_access(event).await?;
```

### Layer 4: Behavioral Anomaly Detection

**Location**: `crates/aof-runtime/src/credential_anomaly.rs`

**Purpose**: Establish behavioral baselines, detect unusual access patterns.

**Algorithm**:

Anomaly score = frequency_score + volume_score + time_score + burst_score (capped at 1.0)

| Component | Weight | Trigger Condition | Example |
|-----------|--------|-------------------|---------|
| Frequency | 0.0-0.4 | Interval < 10% of baseline mean | Access every 10s when baseline is 1 hour |
| Volume | 0.0-0.3 | Daily accesses > 3x baseline mean | 50 accesses/day when baseline is 10/day |
| Time-of-day | 0.0-0.2 | Access outside established active hours | Access at 3am when baseline is 9am-6pm |
| Burst | 0.0-0.3 | >5 accesses within 60 seconds | 10 accesses in 10 seconds |

**Lifecycle**:

1. **Learning Mode** (first 7 days or >= 10 samples):
   - All accesses score 0.0 (no false positives)
   - Baselines established after 10+ accesses per agent+credential pair

2. **Operational Mode**:
   - Score each access against baseline
   - Recommended actions:
     - 0.0-0.5: Allow
     - 0.5-0.7: Log
     - 0.7-0.8: Alert
     - 0.8-0.95: RequireApproval
     - >0.95: Block

**Baseline Storage**: In-memory (RwLock<HashMap>). For production, persist to disk or database.

**Tuning**:

- Adjust component weights in `AnomalyDetector::score_access()`
- Modify thresholds in `AnomalyAction::from_score()`
- Extend learning period beyond 7 days if needed

## Implementation Notes

### Performance Impact

- **Seccomp overhead**: <5% (typically 1-3% for well-designed profiles)
- **Audit logging**: <1% (async file I/O, buffered writes)
- **Anomaly detection**: <0.5% (in-memory baseline lookups)

**Measurement**: See `tests/sandbox_escape.rs::test_seccomp_overhead_estimate` for validation approach.

### Error Handling

All security errors use `AofError::security()` for consistent handling:

```rust
if anomaly_score > 0.95 {
    return Err(AofError::security(format!(
        "Credential access blocked: agent={}, score={}, reasons={:?}",
        agent_id, anomaly_score, reasons
    )));
}
```

### Logging

Security events use structured logging with appropriate levels:

- **INFO**: Profile applied, capability config
- **DEBUG**: Security args, individual accesses
- **WARN**: Sequence gaps, anomaly scores > 0.7
- **ERROR**: Blocked accesses, security violations

## Security Test Suite

**Location**: `crates/aof-runtime/tests/`

### Sandbox Escape Tests (`sandbox_escape.rs`)

| Test | Validates |
|------|-----------|
| `test_seccomp_blocks_ptrace` | ptrace syscall blocked |
| `test_seccomp_blocks_mount` | mount syscall blocked |
| `test_seccomp_blocks_module_loading` | init_module, finit_module blocked |
| `test_seccomp_blocks_namespace_manipulation` | setns, unshare blocked |
| `test_seccomp_blocks_bpf` | bpf syscall blocked |
| `test_capabilities_drop_all_default` | Default drops all capabilities |
| `test_capability_allowlist_per_tool` | Tool-specific allowlists work |
| `test_readonly_profile_minimal_syscalls` | readonly profile allows ~15 syscalls |
| `test_profile_selection_by_tool_name` | Tools map to correct profiles |
| `test_seccomp_overhead_estimate` | Profiles are parseable and generate valid Docker args |

### Credential Audit Tests (`credential_audit.rs`)

| Test | Validates |
|------|-----------|
| `test_credential_detection_kubectl` | kubectl detected as Kubernetes |
| `test_credential_detection_aws` | aws detected as AWS |
| `test_audit_log_sequence_numbers` | 100 events have monotonic sequence |
| `test_audit_log_tamper_detection` | Deleted events create sequence gaps |
| `test_anomaly_score_normal_access` | Normal patterns score <= 0.3 |
| `test_anomaly_score_frequency_spike` | 10x frequency scores >= 0.3 |
| `test_anomaly_score_off_hours` | Time-of-day tracking works |
| `test_anomaly_blocks_extreme_score` | Score > 0.95 returns Block action |
| `test_learning_mode_no_blocks` | Learning mode always scores 0.0 |
| `test_audit_event_json_format` | Events serialize to expected JSON |

**Running Tests**:

```bash
# All security tests
cargo test -p aof-runtime --test sandbox_escape --test credential_audit

# Specific test
cargo test -p aof-runtime --test sandbox_escape test_seccomp_blocks_ptrace
```

## Future Enhancements

### High Priority

1. **Persistent baseline storage**: Save baselines to SQLite/PostgreSQL
2. **Real-time alerting**: Integrate with PagerDuty/Slack for score > 0.8
3. **Credential rotation detection**: Distinguish rotation from exfiltration
4. **ML-based anomaly detection**: Train models on historical access patterns

### Medium Priority

1. **Seccomp profile generation**: Auto-generate profiles via trace analysis
2. **Audit log encryption**: Encrypt audit logs at rest
3. **Federated anomaly detection**: Share baselines across daemon instances
4. **Resource exhaustion detection**: Track CPU/memory per agent

### Low Priority

1. **SELinux/AppArmor profiles**: Additional MAC layer
2. **Network egress filtering**: Block unauthorized network access
3. **Filesystem access auditing**: Track file reads beyond credentials
4. **Syscall trace analysis**: Detect novel attack patterns

## References

- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
- [Seccomp Profile Format](https://docs.docker.com/engine/security/seccomp/)
- [Linux Capabilities](https://man7.org/linux/man-pages/man7/capabilities.7.html)
- [Container Escape Techniques](https://www.crowdstrike.com/cybersecurity-101/container-security/)

## Support

For security issues, see AOF's security policy (SECURITY.md in repository root).

For implementation questions, file a GitHub issue with the `security` label.
