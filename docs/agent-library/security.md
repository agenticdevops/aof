---
sidebar_position: 6
sidebar_label: Security
---

# Security Agents

Production-ready agents for security scanning, compliance, and threat detection.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [security-scanner](#security-scanner) | Vulnerability scanning | trivy_*, kubectl |
| [compliance-auditor](#compliance-auditor) | Compliance checking | kubectl |
| [secret-rotator](#secret-rotator) | Secret management | kubectl |
| [vulnerability-patcher](#vulnerability-patcher) | Patch vulnerabilities | trivy_*, kubectl |
| [threat-hunter](#threat-hunter) | Proactive threat detection | trivy_*, aws_iam |

## security-scanner

Performs comprehensive security scans across container images, configurations, and infrastructure.

### Usage

```bash
# Scan container images
aofctl run agent library://security/security-scanner \
  --prompt "Scan all images in namespace production for vulnerabilities"

# Scan Kubernetes configurations
aofctl run agent library://security/security-scanner \
  --prompt "Check for security misconfigurations in my K8s manifests"
```

### Capabilities

- Container image vulnerability scanning
- Kubernetes configuration auditing
- Secret exposure detection
- RBAC analysis
- Network policy review
- Compliance validation (CIS, NSA)

### Tools Used
- `trivy_image_scan` - Scan container images
- `trivy_fs_scan` - Scan filesystems
- `trivy_config_scan` - Scan IaC configs
- `kubectl` - Kubernetes operations

### Example Output

```markdown
## Security Scan Report: namespace production

### Summary

| Severity | Count | Fixed Available |
|----------|-------|-----------------|
| CRITICAL | 3 | 3 (100%) |
| HIGH | 12 | 10 (83%) |
| MEDIUM | 28 | 20 (71%) |
| LOW | 45 | 15 (33%) |

### Critical Vulnerabilities

#### CVE-2024-1234: Remote Code Execution
- **Image**: api-server:v2.0.3
- **Package**: openssl 3.0.1
- **Fixed In**: openssl 3.0.12
- **CVSS**: 9.8

**Impact**: Allows remote attackers to execute arbitrary code
via malformed TLS handshake.

**Remediation**:
```dockerfile
FROM node:20-alpine
# Update base image to get patched openssl
RUN apk update && apk upgrade openssl
```

#### CVE-2024-5678: Privilege Escalation
- **Image**: worker:v1.5.0
- **Package**: sudo 1.9.5
- **Fixed In**: sudo 1.9.14
- **CVSS**: 9.1

### Configuration Issues

| Resource | Issue | Severity | Fix |
|----------|-------|----------|-----|
| deploy/api | Container runs as root | HIGH | Add runAsNonRoot: true |
| deploy/worker | No resource limits | MEDIUM | Add limits.memory |
| pod/debug | hostNetwork: true | HIGH | Remove hostNetwork |

### Secrets Exposure

| Finding | Location | Risk |
|---------|----------|------|
| Hardcoded AWS key | deployment.yaml:45 | CRITICAL |
| Database password | configmap/app-config | HIGH |

### Recommendations

1. **Immediate** (Critical CVEs):
   - Update api-server base image
   - Patch openssl in all images

2. **This Week** (High severity):
   - Enable runAsNonRoot for all containers
   - Move secrets to Kubernetes Secrets
   - Remove debug pod with hostNetwork

3. **This Month** (Medium):
   - Add network policies
   - Enable pod security standards
   - Implement image signing
```

---

## compliance-auditor

Audits infrastructure against compliance frameworks and standards.

### Usage

```bash
# Run CIS benchmark
aofctl run agent library://security/compliance-auditor \
  --prompt "Audit Kubernetes cluster against CIS benchmark"

# Check specific compliance
aofctl run agent library://security/compliance-auditor \
  --prompt "Verify PCI-DSS compliance for payment namespace"
```

### Capabilities

- CIS Kubernetes Benchmark
- NSA/CISA Hardening Guidelines
- PCI-DSS controls
- SOC2 requirements
- HIPAA safeguards
- Custom policy enforcement

### Tools Used
- `kubectl` - Kubernetes auditing

### Example Output

```markdown
## Compliance Audit: CIS Kubernetes Benchmark v1.8

### Overall Score: 72/100

### Control Plane (Score: 85%)

| Control | Status | Details |
|---------|--------|---------|
| 1.1.1 API Server audit | PASS | Audit logging enabled |
| 1.1.2 etcd encryption | PASS | Secrets encrypted at rest |
| 1.2.1 Anonymous auth | PASS | Disabled |
| 1.2.2 Basic auth | PASS | Disabled |
| 1.2.3 Token auth | FAIL | Token file exists |
| 1.2.4 Kubelet HTTPS | PASS | Enabled |

### Worker Nodes (Score: 78%)

| Control | Status | Details |
|---------|--------|---------|
| 4.1.1 Kubelet auth | PASS | Webhook enabled |
| 4.1.2 Read-only port | FAIL | Port 10255 open |
| 4.2.1 File permissions | WARN | Some files 644 |
| 4.2.2 Ownership | PASS | root:root |

### Policies (Score: 65%)

| Control | Status | Details |
|---------|--------|---------|
| 5.1.1 RBAC enabled | PASS | RBAC authorization |
| 5.1.2 Least privilege | FAIL | 3 cluster-admin bindings |
| 5.2.1 Pod Security | WARN | Not enforced globally |
| 5.3.1 Network Policies | FAIL | 5 namespaces missing |

### Failed Controls

#### 1.2.3 Token Auth File
**Risk**: Static tokens can be compromised
**Fix**: Disable token auth file, use OIDC

```yaml
# API server config
- --token-auth-file=  # Remove this line
```

#### 4.1.2 Read-Only Port
**Risk**: Exposes node information
**Fix**: Disable read-only port

```yaml
# Kubelet config
readOnlyPort: 0
```

### Remediation Priority

1. **Critical**: Remove cluster-admin bindings
2. **High**: Disable kubelet read-only port
3. **Medium**: Enable Pod Security Standards
4. **Low**: Add network policies to remaining namespaces
```

---

## secret-rotator

Manages and rotates secrets across the infrastructure.

### Usage

```bash
# Check secret age
aofctl run agent library://security/secret-rotator \
  --prompt "List all secrets older than 90 days"

# Rotate secrets
aofctl run agent library://security/secret-rotator \
  --prompt "Rotate database credentials for production"
```

### Capabilities

- Secret age tracking
- Automated rotation workflows
- Integration with secret managers
- Rotation verification
- Audit logging
- Rollback support

### Tools Used
- `kubectl` - Kubernetes secrets

### Example Output

```markdown
## Secret Rotation Report

### Secret Age Analysis

| Namespace | Secret | Age | Policy | Status |
|-----------|--------|-----|--------|--------|
| production | db-credentials | 120 days | 90 days | OVERDUE |
| production | api-keys | 45 days | 90 days | OK |
| production | tls-cert | 340 days | 365 days | WARN |
| staging | db-credentials | 95 days | 90 days | OVERDUE |

### Rotation Required

#### db-credentials (production)
**Age**: 120 days (30 days overdue)
**Type**: Database credentials
**Used By**: api-server, worker, scheduler

**Rotation Plan**:
1. Generate new credentials in database
2. Update Kubernetes secret
3. Rolling restart affected deployments
4. Verify connectivity
5. Revoke old credentials

**Commands**:
```bash
# Step 1: Generate new password
NEW_PASS=$(openssl rand -base64 32)

# Step 2: Update database
psql -c "ALTER USER appuser PASSWORD '$NEW_PASS'"

# Step 3: Update secret
kubectl create secret generic db-credentials \
  --from-literal=password=$NEW_PASS \
  --dry-run=client -o yaml | kubectl apply -f -

# Step 4: Rolling restart
kubectl rollout restart deployment/api-server -n production
kubectl rollout restart deployment/worker -n production

# Step 5: Verify
kubectl exec deployment/api-server -- psql -c "SELECT 1"
```

### Rotation History

| Secret | Last Rotated | Rotated By | Status |
|--------|--------------|------------|--------|
| api-keys | 2024-11-05 | automation | Success |
| tls-cert | 2024-01-15 | manual | Success |
| db-credentials | 2024-08-22 | manual | Success |

### Recommendations

1. Enable automated rotation for database credentials
2. Set up alerts for secrets approaching expiry
3. Implement external secret management (Vault, AWS SM)
```

---

## vulnerability-patcher

Automates vulnerability patching and remediation.

### Usage

```bash
# List patchable vulnerabilities
aofctl run agent library://security/vulnerability-patcher \
  --prompt "What critical vulnerabilities can be patched automatically?"

# Generate patch plan
aofctl run agent library://security/vulnerability-patcher \
  --prompt "Create patch plan for CVE-2024-1234"
```

### Capabilities

- Vulnerability assessment
- Patch availability checking
- Automated patching workflows
- Impact analysis
- Rollback preparation
- Patch verification

### Tools Used
- `trivy_image_scan` - Vulnerability scanning
- `trivy_fs_scan` - Filesystem scanning
- `kubectl` - Kubernetes operations

### Example Output

```markdown
## Vulnerability Patch Plan

### Target: CVE-2024-1234 (Critical)

**Vulnerability**: OpenSSL Remote Code Execution
**CVSS Score**: 9.8
**Affected Images**: 5
**Affected Deployments**: 8

### Affected Resources

| Image | Current | Fixed | Deployments |
|-------|---------|-------|-------------|
| api:v2.0.3 | openssl 3.0.1 | openssl 3.0.12 | 3 |
| worker:v1.5.0 | openssl 3.0.1 | openssl 3.0.12 | 2 |
| scheduler:v1.2.1 | openssl 3.0.1 | openssl 3.0.12 | 1 |
| cron:v1.0.5 | openssl 3.0.1 | openssl 3.0.12 | 1 |
| tools:v2.1.0 | openssl 3.0.1 | openssl 3.0.12 | 1 |

### Patch Strategy

**Option 1: Base Image Update (Recommended)**
- Update Alpine base image to latest
- Rebuilds all images
- Estimated time: 30 minutes
- Risk: Low (same app code)

**Option 2: Direct Package Update**
- Add `RUN apk upgrade openssl` to Dockerfiles
- Selective patching
- Estimated time: 45 minutes
- Risk: Medium (untested combination)

### Execution Plan

**Phase 1: Build Patched Images**
```bash
# Update base images
docker build -t api:v2.0.4-security .
docker build -t worker:v1.5.1-security .
```

**Phase 2: Test in Staging**
```bash
kubectl set image deployment/api api=api:v2.0.4-security -n staging
# Run integration tests
./scripts/integration-test.sh
```

**Phase 3: Production Rollout**
```bash
# Canary deployment
kubectl set image deployment/api api=api:v2.0.4-security -n production
kubectl rollout status deployment/api -n production
```

**Phase 4: Verification**
```bash
# Verify patch applied
trivy image api:v2.0.4-security --severity CRITICAL
# Expected: No CVE-2024-1234
```

### Rollback Plan
```bash
kubectl rollout undo deployment/api -n production
```

### Timeline
- Build & Test: 1 hour
- Staging Validation: 2 hours
- Production Rollout: 1 hour
- Total: 4 hours
```

---

## threat-hunter

Proactively hunts for security threats and anomalies.

### Usage

```bash
# Hunt for threats
aofctl run agent library://security/threat-hunter \
  --prompt "Look for signs of compromise in the production cluster"

# Analyze suspicious activity
aofctl run agent library://security/threat-hunter \
  --prompt "Investigate unusual IAM activity in the last 24 hours"
```

### Capabilities

- Anomaly detection
- Threat indicator analysis
- Security posture assessment
- Audit log analysis
- IAM permission review
- Network traffic analysis

### Tools Used
- `trivy_image_scan` - Vulnerability scanning
- `trivy_fs_scan` - Malware detection
- `aws_iam` - IAM analysis
- `gcp_iam` - GCP IAM analysis
- `kubectl` - Kubernetes audit

### Example Output

```markdown
## Threat Hunting Report

### Hunt Parameters
- **Scope**: Production cluster + AWS account
- **Timeframe**: Last 7 days
- **Focus Areas**: IAM, Container runtime, Network

### Findings

#### HIGH: Suspicious IAM Activity
**Indicator**: Unusual API calls from unknown IP

| Time | Action | Principal | Source IP | Risk |
|------|--------|-----------|-----------|------|
| Dec 19 03:42 | CreateAccessKey | admin | 203.0.113.50 | HIGH |
| Dec 19 03:43 | AttachUserPolicy | admin | 203.0.113.50 | HIGH |
| Dec 19 03:45 | AssumeRole | admin | 203.0.113.50 | HIGH |

**Analysis**:
- Source IP not in known IP ranges
- Activity occurred outside business hours
- Multiple privilege escalation attempts

**Recommendation**:
1. Revoke access key created at 03:42
2. Review all changes made by this session
3. Enable MFA for admin account

#### MEDIUM: Container with Crypto Mining Binary
**Indicator**: Suspicious binary in running container

| Pod | Container | Binary | Hash |
|-----|-----------|--------|------|
| debug-pod | alpine | /tmp/miner | abc123... |

**Analysis**:
- Binary matches known cryptominer signature
- Pod running in privileged mode
- No resource limits set

**Recommendation**:
1. Kill the pod immediately
2. Investigate how binary was introduced
3. Review pod creation audit logs

#### LOW: Excessive Failed Login Attempts
**Indicator**: Brute force pattern detected

| Source | Target | Attempts | Timeframe |
|--------|--------|----------|-----------|
| 10.0.5.23 | api-server:443 | 1,247 | 1 hour |

**Analysis**:
- Internal IP (compromised pod?)
- Targeting admin endpoints
- Attempts from single source

**Recommendation**:
1. Block source IP at network policy level
2. Investigate source pod
3. Enable rate limiting on auth endpoints

### Security Posture Score: 6.5/10

| Category | Score | Issues |
|----------|-------|--------|
| IAM | 5/10 | Admin key compromise possible |
| Container | 7/10 | 1 suspicious container found |
| Network | 8/10 | Brute force detected |
| Secrets | 6/10 | 2 overdue rotations |

### Immediate Actions Required

1. [ ] Revoke suspicious IAM access key
2. [ ] Kill debug-pod with miner binary
3. [ ] Block brute force source IP
4. [ ] Enable CloudTrail alerts for IAM changes
```

---

## Environment Setup

```bash
# Trivy (local scanning)
# No auth needed for public registries

# AWS IAM analysis
export AWS_REGION=us-east-1
export AWS_PROFILE=security-audit

# GCP IAM analysis
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/credentials.json

# Kubernetes
export KUBECONFIG=~/.kube/config
```

## Next Steps

- [Cloud Agents](./cloud.md) - Cloud operations
- [Kubernetes Agents](./kubernetes.md) - K8s debugging
- [Incident Response Tutorial](../tutorials/incident-response.md)
