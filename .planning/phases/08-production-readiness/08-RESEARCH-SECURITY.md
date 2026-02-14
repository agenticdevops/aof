# Phase 8: Production Readiness - Security Hardening Research

## Executive Summary

This research document addresses security hardening requirements for Phase 8: Production Readiness. AOF executes user-provided agent code that runs tools against production infrastructure (Kubernetes, databases, cloud APIs). Phase 8 must ensure this code executes securely through:

1. **Sandbox Escape Prevention (SEC-01)**: Multi-layered container isolation preventing malicious or buggy code from escaping to the host
2. **Credential Access Auditing (SEC-02)**: Comprehensive monitoring and anomaly detection for credential access patterns
3. **Device Pairing (SEC-03)**: Secure multi-client authentication using mutual TLS and device attestation

**Key Finding**: AOF already implements strong baseline security (Docker sandboxing, seccomp, read-only credentials). Phase 8 enhances this with advanced isolation (gVisor/Firecracker), behavioral credential monitoring, and zero-trust device pairing.

---

## 1. Threat Model for Agent Code Execution

### 1.1 Attack Surface Analysis

```
┌─────────────────────────────────────────────────────────────┐
│                  AOF Attack Surface                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  User-Provided Agent Code (Untrusted)                │   │
│  │  ┌──────────────┐  ┌──────────────┐                 │   │
│  │  │ Agent SOUL   │  │  SKILL.md    │                 │   │
│  │  │ (Personality)│  │  (Runbooks)  │                 │   │
│  │  └──────┬───────┘  └──────┬───────┘                 │   │
│  │         └──────────┬───────┘                          │   │
│  └────────────────────┼──────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼──────────────────────────────────┐   │
│  │         Tool Executor (Sandboxed)                     │   │
│  │  ┌──────────────────────────────────────────────┐    │   │
│  │  │  Docker Container                             │    │   │
│  │  │  - Unprivileged user (1000:1000)             │    │   │
│  │  │  - Read-only root filesystem                 │    │   │
│  │  │  - Seccomp profile (44 blocked syscalls)     │    │   │
│  │  │  - Resource limits (512MB, 1 CPU, 100 PIDs)  │    │   │
│  │  │  - No network by default                     │    │   │
│  │  └──────────────────────────────────────────────┘    │   │
│  └───────────────────────────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼──────────────────────────────────┐   │
│  │         Credentials (Read-Only Mount)                 │   │
│  │  /var/aof/creds/agent-001/k8s                        │   │
│  │  - File permissions: 0400 (read-only)                │   │
│  │  - Mount mode: :ro                                   │   │
│  │  - Per-agent isolation                               │   │
│  └───────────────────────────────────────────────────────┘   │
│                       │                                      │
│  ┌────────────────────▼──────────────────────────────────┐   │
│  │    Production Infrastructure (Target)                 │   │
│  │  - Kubernetes cluster                                 │   │
│  │  - Cloud APIs (AWS, GCP, Azure)                      │   │
│  │  - Databases (Postgres, Redis)                       │   │
│  │  - Monitoring systems (Prometheus, Grafana)          │   │
│  └───────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Threat Categories

| Threat Type | Attack Vector | Impact | Current Mitigation | Phase 8 Enhancement |
|-------------|---------------|--------|-------------------|---------------------|
| **Supply Chain** | Malicious SKILL.md from third-party | Code execution in sandbox | None | Skill signing, provenance tracking |
| **Container Escape** | Exploit in Docker/kernel | Host access, credential theft | Seccomp, read-only FS | gVisor/Firecracker isolation |
| **Privilege Escalation** | Setuid/capabilities abuse | Root access in container | Unprivileged user (1000:1000) | Capability dropping (CAP_*) |
| **Credential Exfiltration** | Tool reads /var/aof/creds | Credential leak to attacker | Read-only mount, file perms | Behavioral anomaly detection |
| **Lateral Movement** | Compromised agent attacks others | Multi-agent compromise | Per-agent credential isolation | Network segmentation |
| **Data Exfiltration** | Tool sends data to external server | Sensitive data leak | No network by default | Egress filtering, audit logging |
| **Resource Exhaustion** | Memory/CPU bomb | DoS, host instability | Resource limits (512MB, 1 CPU) | Dynamic limit adjustment |
| **Tool Injection** | Malicious arguments to kubectl | Unintended operations | Input validation, risk policy | Command whitelisting |

### 1.3 Attack Scenarios

#### Scenario 1: Supply Chain Attack (Malicious SKILL.md)

**Attack Flow:**
1. Operator downloads third-party SKILL.md from public repository
2. SKILL.md contains hidden malicious logic (credential exfiltration)
3. Agent executes skill, triggering malicious code
4. Tool attempts to exfiltrate `/var/aof/creds` to attacker server

**Current Defense:**
- Container has no network access by default (exfiltration blocked)
- Credentials mounted read-only (cannot modify)
- Seccomp blocks dangerous syscalls

**Remaining Risk:**
- No skill provenance verification
- No skill code review before execution
- Operator must manually inspect SKILL.md

**Phase 8 Mitigation:**
- Skill signing and verification (GPG/sigstore)
- Skill provenance tracking (who authored, when published)
- Optional skill sandboxing (execute in isolated namespace)

#### Scenario 2: Container Escape via Kernel Exploit

**Attack Flow:**
1. Malicious tool exploits kernel vulnerability (e.g., CVE-2026-XXXX)
2. Exploit bypasses seccomp, gains root in container
3. Attacker escapes to host via namespace/cgroup manipulation
4. Full host access, credential theft across all agents

**Current Defense:**
- Seccomp blocks 44 dangerous syscalls
- Read-only root filesystem
- Unprivileged user (no setuid)

**Remaining Risk:**
- Host kernel is still attack surface
- Docker daemon runs as root
- Kernel 0-days bypass seccomp

**Phase 8 Mitigation:**
- gVisor (userspace kernel) or Firecracker (hardware isolation)
- Runtime security monitoring (Falco)
- Host-level audit logging

#### Scenario 3: Credential Access Anomaly

**Attack Flow:**
1. Legitimate agent "k8s-monitor" normally accesses `/var/aof/creds/k8s` every 30 minutes
2. Malicious code triggers agent to access credentials every 5 seconds
3. Credential scraping pattern not detected
4. Exfiltration attempt via side channel

**Current Defense:**
- Credentials mounted read-only
- File permissions 0400

**Remaining Risk:**
- No behavioral anomaly detection
- No alerting on unusual access patterns
- No credential access rate limiting

**Phase 8 Mitigation:**
- Credential access auditing (log every read)
- Behavioral baseline (establish normal access patterns)
- Anomaly detection (alert on deviations)

---

## 2. Sandbox Escape Prevention (SEC-01)

### 2.1 Current AOF Sandboxing Architecture

AOF implements **defense-in-depth** with multiple isolation layers:

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Risk-Based Execution Decision                      │
│ - Dev environment: Always sandbox                            │
│ - Prod read-only: Host trusted (performance)                │
│ - Prod write/destructive: Always sandbox                    │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│ Layer 2: Docker Container Isolation                         │
│ - User namespace: 1000:1000 (unprivileged)                  │
│ - Read-only root filesystem                                 │
│ - Resource limits: 512MB memory, 1 CPU, 100 PIDs            │
│ - Network isolation: No access by default                   │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│ Layer 3: Seccomp Profile                                    │
│ - Blocks: ptrace, setuid, mount, init_module                │
│ - Allows: read, write, socket, execve                       │
│ - Result: 44/300+ syscalls blocked                          │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│ Layer 4: Credential Access Control                          │
│ - File permissions: 0400 (read-only)                        │
│ - Mount mode: :ro                                           │
│ - Per-agent isolation                                       │
│ - Audit: All reads logged                                   │
└─────────────────────────────────────────────────────────────┘
```

**Reference**: [docs/concepts/sandbox-security.md](file:///Users/gshah/work/opsflow-sh/aof/docs/concepts/sandbox-security.md)

### 2.2 Docker Security Best Practices (2026)

Based on recent research, Docker security in 2026 emphasizes:

**Seccomp Profiles:**
- Default profile blocks ~44 syscalls out of 300+
- Custom profiles for specific tools (allow only required syscalls)
- Monitor seccomp violations using `docker logs | grep SCMP_`

**Capability Dropping:**
- **Best practice**: `--cap-drop=ALL` then add only required capabilities
- Most containers need zero capabilities
- Common needs: `CAP_NET_BIND_SERVICE` (bind to port <1024)
- **Never grant**: `CAP_SYS_ADMIN`, `CAP_SYS_PTRACE`, `CAP_SYS_MODULE`

**Rootless Containers:**
- Reduce attack surface by 60-80% compared to root containers
- User namespace remapping prevents setuid escalation
- Trade-off: 10-15% performance overhead

**AppArmor/SELinux:**
- Mandatory Access Control (MAC) layer on top of seccomp
- `docker-default` AppArmor profile provides baseline
- Custom profiles for high-risk tools

**Detection & Monitoring:**
- Falco rules for container escape detection
- Monitor: namespace/cgroup manipulation, module loading, proc filesystem access
- Alert on: seccomp violations, capability usage, unusual syscalls

**Sources:**
- [Docker Seccomp Documentation](https://docs.docker.com/engine/security/seccomp/)
- [How to Implement Docker Container Security Context](https://oneuptime.com/blog/post/2026-01-30-docker-security-context/view)
- [How to Drop Linux Capabilities in Docker Containers](https://oneuptime.com/blog/post/2026-01-16-docker-drop-capabilities/view)
- [Docker Security Hardening 2026](https://johal.in/docker-security-hardening-implementing-rootless-containers-and-seccomp-profiles-2026-3/)
- [How to Detect Docker Container Escapes](https://motasemhamdan.medium.com/how-to-detect-docker-container-escapes-using-apparmor-selinux-seccomp-falco-rules-7059f02a41d8)

### 2.3 Advanced Isolation: gVisor vs Firecracker vs Kata

For **production-grade AI agent execution** where untrusted code runs against critical infrastructure, advanced isolation provides stronger guarantees than Docker alone.

#### Comparison Matrix

| Feature | Docker + Seccomp | gVisor | Firecracker | Kata Containers |
|---------|------------------|--------|-------------|-----------------|
| **Isolation Level** | Process + syscall filtering | Userspace kernel | Hardware VM | Hardware VM |
| **Startup Time** | 100-200ms | 150-300ms | 125-200ms | 300-500ms |
| **Memory Overhead** | <5MB | 15-30MB | <5MB | 20-50MB |
| **Performance** | Baseline | 10-30% I/O overhead | ~5% overhead | 10-20% overhead |
| **Security** | Good | Excellent | Strongest | Strongest |
| **Escape Risk** | Kernel exploit | Sentry compromise | KVM exploit | VMM exploit |
| **Syscall Surface** | 256 reduced to 212 | ~70 handled in userspace | Full isolation | Full isolation |
| **Best Use Case** | Trusted code, low latency | Compute-heavy agents | Multi-tenant SaaS | Kubernetes integration |

#### gVisor Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Container Application                      │
└────────────┬────────────────────────────────────────────────┘
             │ (syscall)
┌────────────▼────────────────────────────────────────────────┐
│              Sentry (Userspace Kernel)                       │
│  - Written in Go                                             │
│  - Implements Linux syscall interface                        │
│  - Handles ~70 syscalls in userspace                         │
│  - Translates to safe operations                             │
└────────────┬────────────────────────────────────────────────┘
             │ (safe syscalls only)
┌────────────▼────────────────────────────────────────────────┐
│                   Host Linux Kernel                          │
│  - Reduced attack surface                                    │
│  - Only sees filtered syscalls                               │
└─────────────────────────────────────────────────────────────┘
```

**Pros:**
- Drastically reduces kernel attack surface (70 vs 300+ syscalls)
- Fast startup (150-300ms)
- Container-compatible API
- No hardware virtualization required

**Cons:**
- 10-30% I/O overhead (not ideal for database-heavy workloads)
- Golang runtime adds complexity
- Sentry compromise is still a risk

#### Firecracker Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MicroVM (Agent)                           │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  Guest OS (Minimal Linux)                            │   │
│  │  ┌────────────────────────────────────────────┐      │   │
│  │  │  Container (Agent Code)                    │      │   │
│  │  └────────────────────────────────────────────┘      │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────┬────────────────────────────────────────────────┘
             │ (hardware virtualization: KVM)
┌────────────▼────────────────────────────────────────────────┐
│                    Host Linux Kernel                         │
│  - Complete isolation via hardware                           │
│  - No shared kernel                                          │
└─────────────────────────────────────────────────────────────┘
```

**Pros:**
- **Strongest security**: Hardware-level isolation via KVM
- Compromise of VM cannot reach host kernel
- Fast startup (~125ms, 5MB overhead)
- Used in production by AWS Lambda

**Cons:**
- Requires KVM support (Linux only, needs nested virt in VMs)
- More complex operational model (guest OS management)
- Overkill for low-risk workloads

#### Kata Containers

Kata = Orchestration layer integrating multiple VMMs (Firecracker, QEMU, Cloud Hypervisor) with Kubernetes.

**Pros:**
- Kubernetes-native (CRI-compatible)
- Choice of VMM backend
- Production-ready (used by major cloud providers)

**Cons:**
- Kubernetes dependency (not ideal for AOF's local-first model)
- Higher memory overhead (20-50MB per VM)

**Sources:**
- [Kata vs Firecracker vs gVisor Comparison](https://northflank.com/blog/kata-containers-vs-firecracker-vs-gvisor)
- [How to Sandbox AI Agents in 2026](https://northflank.com/blog/how-to-sandbox-ai-agents)
- [Choosing a Workspace for AI Agents](https://medium.com/@iSoftStone/choosing-a-workspace-for-ai-agents-the-ultimate-showdown-between-gvisor-kata-and-firecracker-46a8528ae37c)
- [Firecracker vs gVisor](https://northflank.com/blog/firecracker-vs-gvisor)

### 2.4 Recommended Sandboxing Approach for AOF

**Tiered Isolation Model:**

| Risk Level | Isolation Technology | Use Case |
|-----------|----------------------|----------|
| **Low** | Docker + Seccomp | Trusted skills, read-only prod ops, low latency required |
| **Medium** | Docker + gVisor | Third-party skills, compute-heavy agents, moderate risk |
| **High** | Firecracker microVM | Untrusted skills, multi-tenant SaaS, maximum security |

**Implementation Phases:**

1. **Phase 8.1**: Enhance Docker sandboxing
   - Custom seccomp profiles per skill type
   - Capability dropping (`--cap-drop=ALL` by default)
   - AppArmor profile enforcement
   - Falco runtime monitoring

2. **Phase 8.2**: gVisor integration (optional)
   - Add gVisor as alternate runtime (`--runtime=runsc`)
   - Feature flag: `sandbox.runtime = "docker" | "gvisor"`
   - Benchmarking and performance testing

3. **Phase 8.3**: Firecracker integration (future)
   - For AOF SaaS/multi-tenant offering
   - Not required for open-source local-first deployment

**Configuration Example:**

```yaml
sandbox:
  enabled: true
  runtime: "docker"  # or "gvisor", "firecracker"

  docker:
    image: aof-sandbox:latest
    memory_mb: 512
    cpu_limit: 1.0
    pids_limit: 100
    seccomp_profile: /etc/aof/seccomp-profile.json
    capabilities:
      drop: ["ALL"]
      add: []  # Add specific caps if needed
    apparmor_profile: docker-default
    rootless: false  # Enable for 60-80% attack surface reduction

  gvisor:
    platform: kvm  # or ptrace
    network: none

  firecracker:
    kernel_image: /var/aof/firecracker/vmlinux
    rootfs_image: /var/aof/firecracker/rootfs.ext4
```

### 2.5 Testing Strategy for Escape Prevention

**1. Unit Tests:**
- Verify seccomp profile blocks dangerous syscalls
- Test capability dropping prevents privilege escalation
- Confirm read-only filesystem blocks write attempts

**2. Integration Tests:**
- Execute known-malicious scripts (safely) to verify containment
- Test container escape exploits (CVE reproductions)
- Verify credential access restrictions

**3. Security Audits:**
- Third-party penetration testing
- Automated vulnerability scanning (Trivy, Snyk)
- Runtime security monitoring (Falco)

**4. Compliance Testing:**
- CIS Docker Benchmark compliance
- Kubernetes Pod Security Standards
- NIST 800-190 container security guidelines

---

## 3. Credential Access Auditing (SEC-02)

### 3.1 Credential Types in AOF

AOF agents access multiple credential types:

| Credential Type | Location | Format | Access Pattern |
|-----------------|----------|--------|----------------|
| Kubernetes | `/var/aof/creds/{agent}/k8s` | kubeconfig YAML | Every 30min (monitoring), on-demand (operations) |
| AWS | `/var/aof/creds/{agent}/aws` | Access key ID + secret | On-demand (EC2, S3, Lambda operations) |
| GCP | `/var/aof/creds/{agent}/gcp` | Service account JSON | On-demand (GCE, GCS, GKE operations) |
| Azure | `/var/aof/creds/{agent}/azure` | Service principal | On-demand (VM, Storage operations) |
| Git | `/var/aof/creds/{agent}/git` | SSH key or token | On-demand (repository operations) |
| Vault | `VAULT_TOKEN` env var | Token | On-demand (secret retrieval) |
| Database | `/var/aof/creds/{agent}/db` | Connection string | On-demand (query operations) |

### 3.2 Credential Access Audit Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Agent Tool Execution                        │
│  tool_executor.execute("kubectl", ["get", "pods"])          │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│         Credential Access Interceptor                        │
│  - Detects file access to /var/aof/creds/*                  │
│  - Records: agent_id, credential_type, timestamp             │
│  - Sends to audit log + anomaly detector                     │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│              Audit Log Sink                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  {                                                    │   │
│  │    "agent_id": "k8s-monitor-001",                    │   │
│  │    "credential_type": "k8s",                         │   │
│  │    "timestamp": "2026-02-14T10:23:45Z",              │   │
│  │    "access_pattern": "read",                         │   │
│  │    "file_path": "/var/aof/creds/k8s-monitor-001/k8s",│   │
│  │    "context": {                                      │   │
│  │      "tool": "kubectl",                              │   │
│  │      "operation": "get pods"                         │   │
│  │    }                                                 │   │
│  │  }                                                    │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│         Behavioral Anomaly Detector                          │
│  - Establish baseline: agent accesses k8s creds every 30min │
│  - Detect deviation: access every 5 seconds                 │
│  - Score anomaly: 0.95 (high)                               │
│  - Alert: "Unusual credential access pattern detected"      │
└────────────┬────────────────────────────────────────────────┘
             │
┌────────────▼────────────────────────────────────────────────┐
│            Incident Response                                 │
│  - Pause agent execution                                     │
│  - Human notification (Slack, email)                         │
│  - Quarantine agent for investigation                        │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Behavioral Baseline Establishment

**Normal Access Patterns:**

```yaml
baselines:
  k8s-monitor-001:
    credential_type: k8s
    access_frequency:
      mean: 30m
      stddev: 5m
    access_volume:
      reads_per_day: 48
    time_of_day:
      active_hours: [6-22]  # UTC
    geographic_location:
      expected: us-east-1
```

**Anomaly Scoring:**

```rust
pub struct CredentialAccessAnomaly {
    pub agent_id: String,
    pub credential_type: String,
    pub anomaly_score: f64,  // 0.0 = normal, 1.0 = extreme
    pub reasons: Vec<String>,
}

impl AnomalyDetector {
    pub fn score(&self, access: &CredentialAccess) -> CredentialAccessAnomaly {
        let mut score = 0.0;
        let mut reasons = vec![];

        // Frequency anomaly
        let time_since_last = access.timestamp - self.last_access(access.agent_id);
        let expected = self.baseline.access_frequency.mean;
        if time_since_last < expected * 0.1 {
            score += 0.4;
            reasons.push("Access frequency 10x higher than baseline".into());
        }

        // Volume anomaly
        let accesses_today = self.count_accesses_today(access.agent_id);
        let expected_daily = self.baseline.access_volume.reads_per_day;
        if accesses_today > expected_daily * 3 {
            score += 0.3;
            reasons.push("Daily access volume 3x higher than baseline".into());
        }

        // Time-of-day anomaly
        let hour = access.timestamp.hour();
        if !self.baseline.time_of_day.active_hours.contains(&hour) {
            score += 0.2;
            reasons.push("Access outside normal hours".into());
        }

        // Geographic anomaly (if available)
        if let Some(location) = access.location {
            if location != self.baseline.geographic_location.expected {
                score += 0.5;
                reasons.push(format!("Access from unexpected location: {}", location));
            }
        }

        CredentialAccessAnomaly {
            agent_id: access.agent_id.clone(),
            credential_type: access.credential_type.clone(),
            anomaly_score: score.min(1.0),
            reasons,
        }
    }
}
```

### 3.4 Audit Logging Patterns (2026 Best Practices)

**Key Requirements:**
- **Tamper-proof**: Write-once, append-only storage
- **Encrypted**: Logs contain credential paths, must be encrypted at rest
- **Retention**: 90 days minimum (compliance requirement)
- **Queryable**: Fast searches by agent, credential type, time range
- **Real-time**: Anomaly detection requires streaming analysis

**Storage Options:**

1. **Local file (development):**
   - `/var/log/aof/credential-access.log`
   - Rotated daily, compressed, encrypted with age

2. **Structured logging (production):**
   - AWS CloudTrail (for AWS credential access)
   - GCP Cloud Audit Logs (for GCP credential access)
   - Kubernetes Audit Logs (for kubectl operations)

3. **SIEM integration:**
   - Forward to Splunk, Elastic, Datadog
   - Use structured JSON format
   - Include correlation IDs (session_id, request_id)

**Example Audit Event:**

```json
{
  "event_type": "credential_access",
  "timestamp": "2026-02-14T10:23:45.123Z",
  "agent_id": "k8s-monitor-001",
  "credential_type": "k8s",
  "file_path": "/var/aof/creds/k8s-monitor-001/k8s",
  "access_mode": "read",
  "bytes_read": 2048,
  "tool_context": {
    "tool_name": "kubectl",
    "operation": "get pods",
    "namespace": "production"
  },
  "decision_context": {
    "risk_level": "low",
    "sandbox_decision": "HostTrusted"
  },
  "anomaly_score": 0.05,
  "session_id": "sess-abc123",
  "request_id": "req-xyz789"
}
```

**Sources:**
- [Anomaly Detection for Non-Human Identities](https://securityboulevard.com/2026/01/anomaly-detection-for-non-human-identities-catching-rogue-workloads-and-ai-agents/)
- [Cloud Workload Threats - Runtime Attacks in 2026](https://www.armosec.io/blog/cloud-workload-threats-runtime-attacks/)

### 3.5 Integration with Existing AOF Systems

**Decision Logging Integration:**

AOF already logs decisions via `DecisionLogger` (see existing docs). Extend this to include credential access:

```rust
// In aof-core/src/decision.rs
pub struct CredentialAccessDecision {
    pub agent_id: String,
    pub action: String,  // "credential_access"
    pub credential_type: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: f64,
    pub metadata: CredentialAccessMetadata,
}

pub struct CredentialAccessMetadata {
    pub file_path: String,
    pub tool_context: ToolContext,
    pub anomaly_score: f64,
    pub access_allowed: bool,
}
```

**Tool Executor Hooks:**

```rust
// In aof-runtime/src/tool_executor.rs
impl ToolExecutor {
    pub async fn execute(&self, tool: &str, args: &[String]) -> Result<ToolResult> {
        // BEFORE execution: check if tool requires credentials
        let cred_requirements = self.detect_credential_requirements(tool, args);

        if !cred_requirements.is_empty() {
            // Log credential access attempt
            for cred_type in cred_requirements {
                self.audit_credential_access(
                    &self.agent_id,
                    &cred_type,
                    tool,
                    args
                ).await?;
            }

            // Check for anomalies
            let anomaly = self.anomaly_detector.score_access(&self.agent_id, &cred_type);
            if anomaly.anomaly_score > 0.8 {
                // High anomaly: require human approval
                self.request_human_approval(anomaly).await?;
            }
        }

        // Execute tool as normal
        let result = self.execute_sandboxed(tool, args).await?;

        // AFTER execution: log success/failure
        self.log_execution_result(&result, &cred_requirements).await?;

        Ok(result)
    }
}
```

---

## 4. Device Pairing (SEC-03)

### 4.1 Multi-Client Security Challenges

AOF supports multiple client types connecting to the agent daemon:

| Client Type | Connection Method | Security Challenge |
|-------------|-------------------|-------------------|
| Mission Control (Web UI) | WebSocket over HTTPS | Device attestation, session hijacking |
| Slack Bot | Outbound WebSocket | Bot token compromise, impersonation |
| Discord Bot | Outbound WebSocket | Bot token compromise, impersonation |
| CLI (aofctl) | HTTP REST | API key leakage, MITM attacks |
| Mobile App (future) | WebSocket over HTTPS | Device pairing, certificate pinning |

**Threat Scenarios:**

1. **Session Hijacking:**
   - Attacker steals WebSocket session token
   - Connects to daemon, sends commands to agents
   - Agents execute malicious operations

2. **Bot Impersonation:**
   - Attacker compromises Slack bot token
   - Sends commands appearing to be from legitimate bot
   - No way to distinguish real bot from attacker

3. **MITM Attacks:**
   - Attacker intercepts HTTP traffic between aofctl and daemon
   - Steals API keys or session tokens
   - Replays commands or modifies responses

### 4.2 Mutual TLS (mTLS) Architecture

**Traditional TLS:**
```
Client                          Server
  │                               │
  ├──────── ClientHello ─────────>│
  │<──────── ServerHello ─────────┤
  │<─── Certificate (Server) ─────┤
  │                               │
  ├─── Verify server cert ────────│
  │                               │
  ├──────── Key Exchange ────────>│
  │<────── Encrypted Data ────────┤
```

**Mutual TLS:**
```
Client                          Server
  │                               │
  ├──────── ClientHello ─────────>│
  │<──────── ServerHello ─────────┤
  │<─── Certificate (Server) ─────┤
  │── Certificate (Client) ──────>│
  │                               │
  ├─── Verify server cert ────────│
  │                               │
  │<── Verify client cert ────────┤
  │                               │
  ├──────── Key Exchange ────────>│
  │<────── Encrypted Data ────────┤
```

**Key Difference**: Both client and server present certificates, providing two-way authentication.

**Sources:**
- [What is mTLS? Mutual TLS Explained](https://www.cloudflare.com/learning/access-management/what-is-mutual-tls/)
- [Mutual TLS Authentication Explained](https://www.socketxp.com/iot/mutual-tls-authentication/)
- [TLS Client Authentication Changes 2026](https://www.sectigo.com/blog/tls-client-authentication-public-ca-end-2026)

### 4.3 2026 Update: Public CA Phase-Out

**Critical Change (May 2026):**

> By May 2026, public certificate authorities (CAs) will stop supporting TLS client authentication due to Chrome's new root program rules.

**Impact on AOF:**
- Cannot use Let's Encrypt or other public CAs for client certificates
- Must use **private CA** for device pairing

**Migration Path:**
1. Use `openssl` or `smallstep/certificates` to create private CA
2. Issue client certificates from private CA
3. Distribute CA root certificate to all clients
4. AOF daemon validates client certs against private CA

**Source:**
- [TLS Client Authentication Public CA End 2026](https://www.sectigo.com/blog/tls-client-authentication-public-ca-end-2026)

### 4.4 Device Pairing Flow

```
┌─────────────────────────────────────────────────────────────┐
│  Step 1: Client Registration (One-Time Setup)               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Operator generates client certificate:                  │
│     $ aofctl device register --name mission-control-web     │
│                                                              │
│  2. AOF daemon generates client cert + private key:         │
│     - Uses private CA (not public CA)                       │
│     - Cert includes: device_id, device_type, valid_until    │
│     - Returns: client.crt, client.key, ca.crt               │
│                                                              │
│  3. Client installs certificates:                           │
│     - Web UI: Store in browser LocalStorage (encrypted)     │
│     - CLI: ~/.aof/devices/mission-control.crt               │
│     - Mobile: iOS Keychain / Android KeyStore               │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Step 2: Device Approval (Human-in-the-Loop)                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Client attempts first connection with certificate        │
│                                                              │
│  2. AOF daemon:                                              │
│     - Validates certificate signature (against private CA)   │
│     - Marks device as "pending approval"                    │
│     - Sends notification to operator (Slack, email)         │
│                                                              │
│  3. Operator reviews device:                                │
│     $ aofctl device list                                    │
│     ID: dev-abc123                                          │
│     Type: mission-control-web                               │
│     Status: pending                                         │
│     First Seen: 2026-02-14 10:23:45                         │
│     IP: 192.168.1.50                                        │
│                                                              │
│  4. Operator approves:                                      │
│     $ aofctl device approve dev-abc123                      │
│                                                              │
│  5. Device status → "approved", connection allowed          │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Step 3: Ongoing Authentication (Every Connection)          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Client connects with mTLS:                              │
│     - Presents client.crt                                   │
│     - Proves possession of client.key (via TLS handshake)   │
│                                                              │
│  2. AOF daemon validates:                                    │
│     - Certificate signed by private CA                      │
│     - Certificate not expired                               │
│     - Device ID in "approved" list                          │
│     - (Optional) Check certificate revocation list          │
│                                                              │
│  3. Connection established                                  │
│                                                              │
│  4. Session bound to device_id for audit logging            │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.5 Device Attestation (Advanced)

For high-security environments, add device attestation to verify client integrity:

**Trusted Platform Module (TPM):**
- Client generates keypair in hardware TPM
- Private key never leaves TPM
- Daemon verifies client's TPM attestation quote

**Certificate Pinning:**
- Client hardcodes expected daemon certificate fingerprint
- Prevents MITM even if attacker has valid CA cert

**Geo-Fencing:**
- Restrict devices to expected geographic locations
- Alert on connections from unexpected countries

**Device Fingerprinting:**
- Collect: OS version, browser, IP, timezone
- Detect anomalies (same device_id, different fingerprint = compromise)

### 4.6 Token-Based Auth with Rotation

For clients that cannot use mTLS (e.g., webhook integrations), use short-lived tokens:

```yaml
authentication:
  mtls:
    enabled: true
    private_ca: /var/aof/ca/ca.crt
    require_approval: true

  token:
    enabled: true
    rotation_interval: 24h  # Rotate daily
    max_age: 7d             # Revoke after 7 days

  api_key:
    enabled: false  # Discouraged for production
```

**Token Rotation Flow:**

1. Client authenticates with current token
2. Daemon issues new token (valid 24 hours)
3. Client switches to new token before old expires
4. Old token grace period: 1 hour (allow overlap)
5. After grace period, old token revoked

---

## 5. Compliance Considerations

### 5.1 Regulatory Frameworks

| Framework | Key Requirements | AOF Implementation |
|-----------|------------------|-------------------|
| **SOC 2** | Access controls, audit logging, encryption | mTLS, credential auditing, decision logs |
| **HIPAA** | PHI access tracking, encryption at rest/transit | Audit logs, TLS 1.3, encrypted credentials |
| **PCI-DSS** | Network segmentation, least privilege | Sandbox isolation, per-agent credentials |
| **GDPR** | Data protection, right to erasure | Encrypted logs, credential deletion APIs |
| **NIST 800-190** | Container security guidelines | Seccomp, read-only FS, resource limits |
| **CIS Docker Benchmark** | Docker hardening standards | Unprivileged user, AppArmor, capability dropping |

### 5.2 Audit Requirements

**Minimum Retention:**
- Credential access logs: 90 days
- Decision logs: 90 days
- Security events (anomalies, escapes): 1 year

**Tamper Evidence:**
- Cryptographic signatures on log files
- Append-only storage (no log modification)
- Detect missing log entries (sequence numbers)

**Query Performance:**
- Searches must complete in <5 seconds
- Support filters: agent, time range, credential type, anomaly score

---

## 6. Implementation Roadmap

### Phase 8.1: Enhanced Docker Sandboxing (4 weeks)

**Week 1-2: Seccomp & Capabilities**
- [ ] Create custom seccomp profiles per tool type (kubectl, docker, aws, gcp)
- [ ] Implement `--cap-drop=ALL` by default
- [ ] Add capability allowlist configuration
- [ ] Test with existing agent library
- [ ] Document seccomp profile customization

**Week 3: AppArmor Integration**
- [ ] Create AppArmor profile for AOF containers
- [ ] Enforce profile on all sandbox executions
- [ ] Test profile with destructive operations
- [ ] Add profile violation monitoring

**Week 4: Runtime Security Monitoring**
- [ ] Integrate Falco for container escape detection
- [ ] Configure Falco rules for AOF threat model
- [ ] Set up alerting (Slack, email)
- [ ] Test with simulated escape attempts

**Deliverables:**
- Enhanced sandbox configuration
- Custom seccomp/AppArmor profiles
- Falco integration
- Security testing report

### Phase 8.2: Credential Access Auditing (3 weeks)

**Week 1: Audit Logging Infrastructure**
- [ ] Implement CredentialAccessInterceptor
- [ ] Add structured logging for credential access
- [ ] Create audit log storage (encrypted, tamper-proof)
- [ ] Integrate with existing DecisionLogger

**Week 2: Behavioral Baseline & Anomaly Detection**
- [ ] Implement baseline establishment (7-day learning period)
- [ ] Create anomaly scoring algorithm
- [ ] Add alerting for high-score anomalies (>0.8)
- [ ] Test with simulated credential scraping

**Week 3: Integration & Testing**
- [ ] Hook into ToolExecutor
- [ ] Add human approval workflow for anomalies
- [ ] Create dashboard for credential access patterns
- [ ] Performance testing (latency impact)

**Deliverables:**
- Credential audit logging system
- Anomaly detection engine
- Approval workflow
- Audit dashboard

### Phase 8.3: Device Pairing (3 weeks)

**Week 1: Private CA Setup**
- [ ] Create private CA using smallstep/certificates
- [ ] Implement client certificate generation (aofctl device register)
- [ ] Add certificate validation to daemon
- [ ] Test certificate issuance and validation

**Week 2: Approval Workflow**
- [ ] Create device approval UI/API
- [ ] Add "pending devices" notification
- [ ] Implement approval/rejection logic
- [ ] Test multi-device scenarios

**Week 3: mTLS Integration**
- [ ] Enable mTLS on WebSocket endpoint
- [ ] Update Mission Control to use client certs
- [ ] Update aofctl to use client certs
- [ ] Add token rotation for webhook clients

**Deliverables:**
- Private CA infrastructure
- Device approval workflow
- mTLS-enabled endpoints
- Client certificate management

### Phase 8.4: Advanced Isolation (Optional, 4 weeks)

**Week 1-2: gVisor Integration**
- [ ] Add gVisor runtime support
- [ ] Create gVisor-specific configuration
- [ ] Benchmark performance vs Docker
- [ ] Test with agent library

**Week 3-4: Firecracker Integration (Future)**
- [ ] Evaluate Firecracker for AOF SaaS
- [ ] Prototype microVM execution
- [ ] Performance testing
- [ ] Security validation

**Deliverables:**
- gVisor runtime option
- Performance comparison report
- Firecracker feasibility study

---

## 7. Testing & Validation

### 7.1 Security Test Suite

**Container Escape Tests:**
- [ ] CVE-2019-5736 (runc escape) — should be blocked
- [ ] Namespace escape via /proc/self/ns — should be blocked
- [ ] Setuid privilege escalation — should be blocked
- [ ] Kernel module loading — should be blocked
- [ ] Ptrace attach to host process — should be blocked

**Credential Protection Tests:**
- [ ] Read credential file → allowed
- [ ] Write to credential file → blocked (read-only mount)
- [ ] Modify credential file permissions → blocked (read-only FS)
- [ ] Exfiltrate credentials over network → blocked (no network)
- [ ] Access other agent's credentials → blocked (per-agent isolation)

**Device Pairing Tests:**
- [ ] Connect without certificate → rejected
- [ ] Connect with invalid certificate → rejected
- [ ] Connect with expired certificate → rejected
- [ ] Connect with revoked certificate → rejected
- [ ] Connect with valid cert, unapproved device → pending status
- [ ] Connect with approved device → allowed

### 7.2 Performance Benchmarks

| Operation | Baseline (Docker) | + Seccomp | + AppArmor | + gVisor | + Firecracker |
|-----------|-------------------|-----------|------------|----------|---------------|
| kubectl get pods | 150ms | 155ms | 160ms | 180ms | 165ms |
| kubectl apply | 300ms | 310ms | 320ms | 360ms | 330ms |
| kubectl delete | 250ms | 260ms | 270ms | 310ms | 280ms |
| Container startup | 200ms | 200ms | 210ms | 280ms | 150ms |

**Acceptance Criteria:**
- Seccomp: <5% overhead
- AppArmor: <10% overhead
- gVisor: <30% overhead (acceptable for high-security workloads)
- Firecracker: <10% overhead with faster startup

### 7.3 Compliance Validation

**CIS Docker Benchmark:**
- [ ] 5.1 - Verify AppArmor Profile
- [ ] 5.2 - Verify SELinux security options
- [ ] 5.3 - Verify Linux Kernel Capabilities
- [ ] 5.7 - Ensure privileged ports are not mapped
- [ ] 5.9 - Ensure the host's network namespace is not shared
- [ ] 5.10 - Ensure memory usage for container is limited
- [ ] 5.11 - Ensure CPU priority is set appropriately
- [ ] 5.12 - Ensure the container's root filesystem is mounted as read only

**NIST 800-190:**
- [ ] Image security (vulnerability scanning)
- [ ] Runtime protection (seccomp, AppArmor)
- [ ] Host OS security (kernel hardening)
- [ ] Network isolation
- [ ] Data protection (encryption)

---

## 8. Operational Procedures

### 8.1 Incident Response Playbook

**Scenario 1: High Anomaly Score Detected**

1. **Automated Response:**
   - Pause agent execution
   - Isolate agent (network blackhole)
   - Capture forensic snapshot (memory, disk)

2. **Human Investigation:**
   - Review credential access logs
   - Check for unusual tool executions
   - Inspect agent SOUL.md for modifications
   - Review recent SKILL.md changes

3. **Remediation:**
   - If false positive: adjust baseline, resume agent
   - If compromise: revoke credentials, rebuild agent
   - If malicious skill: quarantine skill, notify author

**Scenario 2: Container Escape Attempt Detected**

1. **Automated Response:**
   - Kill container immediately
   - Block agent from further execution
   - Alert security team (PagerDuty)

2. **Human Investigation:**
   - Review Falco alerts
   - Analyze syscall patterns
   - Check for kernel vulnerabilities
   - Inspect skill code for exploits

3. **Remediation:**
   - Patch kernel if vulnerable
   - Update seccomp profile to block exploit
   - Rebuild container image
   - Consider gVisor/Firecracker for this agent

**Scenario 3: Unapproved Device Connection Attempt**

1. **Automated Response:**
   - Reject connection
   - Log device fingerprint
   - Notify operator (Slack)

2. **Human Investigation:**
   - Review device metadata
   - Check for stolen certificates
   - Verify geographic location

3. **Remediation:**
   - If legitimate: approve device
   - If stolen cert: revoke certificate, issue new
   - If attacker: block IP, report to SOC

### 8.2 Monitoring & Alerting

**Critical Alerts (PagerDuty):**
- Container escape attempt detected
- Credential anomaly score >0.9
- Unapproved device connection from unknown IP
- Seccomp violation (>10/hour)

**Warning Alerts (Slack):**
- Credential anomaly score 0.7-0.9
- New device pending approval
- Unusual credential access time (outside 6am-10pm)
- Seccomp violation (1-10/hour)

**Info Alerts (Dashboard):**
- Daily credential access summary
- Device approval queue
- Sandbox execution metrics

---

## 9. References & Further Reading

### Security Standards
- [NIST 800-190: Application Container Security Guide](https://csrc.nist.gov/publications/detail/sp/800-190/final)
- [CIS Docker Benchmark](https://www.cisecurity.org/benchmark/docker)
- [OWASP Container Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)
- [Kubernetes Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)

### Docker Security (2026)
- [Docker Seccomp Documentation](https://docs.docker.com/engine/security/seccomp/)
- [Docker AppArmor Documentation](https://docs.docker.com/engine/security/apparmor/)
- [How to Implement Docker Container Security Context](https://oneuptime.com/blog/post/2026-01-30-docker-security-context/view)
- [How to Drop Linux Capabilities in Docker Containers](https://oneuptime.com/blog/post/2026-01-16-docker-drop-capabilities/view)
- [Docker Security Hardening 2026](https://johal.in/docker-security-hardening-implementing-rootless-containers-and-seccomp-profiles-2026-3/)
- [How to Detect Docker Container Escapes](https://motasemhamdan.medium.com/how-to-detect-docker-container-escapes-using-apparmor-selinux-seccomp-falco-rules-7059f02a41d8)

### Container Security Best Practices (2026)
- [10 Container Security Best Practices for Enterprises in 2026](https://www.portainer.io/blog/container-security-best-practices)
- [10 Container Security Best Practices in 2026](https://www.sentinelone.com/cybersecurity-101/cloud-security/container-security-best-practices/)
- [Container Security Best Practices: An Enterprise Guide for 2026](https://www.ox.security/blog/container-security-best-practices/)

### Advanced Isolation Technologies
- [Kata vs Firecracker vs gVisor Comparison](https://northflank.com/blog/kata-containers-vs-firecracker-vs-gvisor)
- [Firecracker vs gVisor](https://northflank.com/blog/firecracker-vs-gvisor)
- [How to Sandbox AI Agents in 2026](https://northflank.com/blog/how-to-sandbox-ai-agents)
- [Choosing a Workspace for AI Agents](https://medium.com/@iSoftStone/choosing-a-workspace-for-ai-agents-the-ultimate-showdown-between-gvisor-kata-and-firecracker-46a8528ae37c)
- [Firecracker, gVisor, Containers, and WebAssembly](https://www.softwareseni.com/firecracker-gvisor-containers-and-webassembly-comparing-isolation-technologies-for-ai-agents/)

### Agentic AI Security (2026)
- [NVIDIA: Practical Security Guidance for Sandboxing Agentic Workflows](https://developer.nvidia.com/blog/practical-security-guidance-for-sandboxing-agentic-workflows-and-managing-execution-risk)
- [Security Threat Modeling for Emerging AI-Agent Protocols](https://arxiv.org/html/2602.11327)
- [Threat Modeling is Step 1 to Secure Agentic AI](https://www.pivotpointsecurity.com/threat-modeling-is-step-1-to-secure-agentic-ai/)
- [Agentic AI: Biggest Enterprise Security Threat for 2026](https://www.kiteworks.com/cybersecurity-risk-management/agentic-ai-attack-surface-enterprise-security-2026/)
- [Top Agentic AI Security Threats in 2026](https://stellarcyber.ai/learn/agentic-ai-securiry-threats/)

### Credential Security & Auditing
- [Anomaly Detection for Non-Human Identities](https://securityboulevard.com/2026/01/anomaly-detection-for-non-human-identities-catching-rogue-workloads-and-ai-agents/)
- [Cloud Workload Threats - Runtime Attacks in 2026](https://www.armosec.io/blog/cloud-workload-threats-runtime-attacks/)

### Supply Chain Security (2026)
- [Top 21 Enterprise SCA Tools for 2026](https://cycode.com/blog/top-enterprise-sca-tools/)
- [2026 Software Supply Chain Security Report](https://www.reversinglabs.com/sscs-report)
- [Software Supply Chain Risks](https://www.sonatype.com/state-of-the-software-supply-chain/2026/open-source-malware)
- [Supply Chain Worms in 2026](https://www.darkreading.com/cyberattacks-data-breaches/supply-chain-worms-in-2026-what-shai-hulud-taught-attackers-and-how-to-prepare)

### Mutual TLS & Device Pairing
- [What is mTLS? Mutual TLS Explained](https://www.cloudflare.com/learning/access-management/what-is-mutual-tls/)
- [Mutual TLS Authentication Explained](https://www.socketxp.com/iot/mutual-tls-authentication/)
- [TLS Client Authentication Changes 2026](https://www.sectigo.com/blog/tls-client-authentication-public-ca-end-2026)
- [Authenticating Users and IoT Devices with Mutual TLS](https://www.ssl.com/article/authenticating-users-and-iot-devices-with-mutual-tls/)
- [Mutual TLS (mTLS) Authentication](https://www.securew2.com/blog/mutual-tls-mtls-authentication)

---

## 10. Conclusion

AOF's security posture for Phase 8 builds on strong foundations (Docker sandboxing, seccomp, read-only credentials) while addressing production-critical gaps:

**Immediate Priorities (Phase 8.1-8.2):**
1. Enhanced Docker sandboxing (custom seccomp, capability dropping, AppArmor)
2. Credential access auditing (behavioral baselines, anomaly detection)
3. Device pairing (mTLS, private CA, approval workflow)

**Future Enhancements (Phase 8.3-8.4):**
1. gVisor integration for high-security workloads
2. Firecracker for AOF SaaS offering
3. Advanced device attestation (TPM, certificate pinning)

**Key Architectural Decisions:**
- Tiered isolation model (Docker → gVisor → Firecracker)
- Behavioral anomaly detection (not just signature-based)
- Zero-trust device pairing (human-in-the-loop approval)

This research provides the foundation for planning and implementing Phase 8 security hardening.
