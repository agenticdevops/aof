---
sidebar_position: 8
---

# Security Examples

Practical examples for common security use cases.

## Vulnerability Scanning

### Scan Container Image

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: image-scanner
spec:
  model: google:gemini-2.5-flash
  tools:
    - trivy_image_scan

  system_prompt: |
    Scan the provided container image and report:
    1. Total vulnerabilities by severity
    2. Critical findings requiring immediate action
    3. Recommended base image updates
```

**Usage:**

```bash
aofctl run agent image-scanner.yaml \
  --input "Scan nginx:1.25-alpine for vulnerabilities"
```

**Expected Output:**

```
## Scan Summary
- Image: nginx:1.25-alpine
- Total: 23 vulnerabilities
- Critical: 0 | High: 2 | Medium: 12 | Low: 9

## High Severity Findings
1. CVE-2024-12345 in openssl 3.0.12
   - Fixed in: 3.0.13
   - Recommendation: Update base image

## Recommended Actions
1. Update to nginx:1.25.3-alpine (fixes 2 HIGH CVEs)
2. Monitor CVE-2024-12346 (no fix available yet)
```

### Scan Project Dependencies

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: dependency-scanner
spec:
  model: google:gemini-2.5-flash
  tools:
    - snyk_test
    - trivy_fs_scan

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"

  system_prompt: |
    Scan project dependencies using both Snyk and Trivy.
    Cross-reference findings and provide unified report.
```

**Usage:**

```bash
aofctl run agent dependency-scanner.yaml \
  --input "Scan /path/to/project for vulnerable dependencies"
```

---

## Secrets Management

### Read and Rotate Secret

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: secret-manager
spec:
  model: google:gemini-2.5-flash
  tools:
    - vault_kv_get
    - vault_kv_put
    - vault_token_lookup

  environment:
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    You manage secrets in Vault.
    - Check secret age before rotation
    - Never display secret values
    - Log all operations
```

**Usage:**

```bash
# Check secret age
aofctl run agent secret-manager.yaml \
  --input "Check when secret/myapp/db was last rotated"

# Rotate secret (agent will generate new value)
aofctl run agent secret-manager.yaml \
  --input "Rotate the API key at secret/myapp/external-api"
```

### Encrypt Sensitive Data

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: data-encryptor
spec:
  model: google:gemini-2.5-flash
  tools:
    - vault_transit_encrypt
    - vault_transit_decrypt

  environment:
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    Encrypt and decrypt sensitive data using Vault Transit.
    Always use the appropriate encryption key for the data type.
```

**Usage:**

```bash
aofctl run agent data-encryptor.yaml \
  --input "Encrypt this SSN: 123-45-6789 using the 'pii' key"
```

---

## Policy Enforcement

### Check Kubernetes Admission

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: k8s-policy-checker
spec:
  model: google:gemini-2.5-flash
  tools:
    - opa_eval

  environment:
    OPA_URL: "${OPA_URL}"

  system_prompt: |
    Evaluate Kubernetes resources against admission policies.
    Explain any violations and how to fix them.
```

**Usage:**

```bash
# Check if a pod would be admitted
aofctl run agent k8s-policy-checker.yaml \
  --input "Check if this pod spec would be admitted" \
  --file pod.yaml
```

**Example Policy Violations:**

```
## Policy Evaluation Result: DENIED

### Violations Found:

1. **runAsNonRoot Required**
   - Policy: containers-must-run-as-non-root
   - Container: app
   - Fix: Add `securityContext.runAsNonRoot: true`

2. **Resource Limits Required**
   - Policy: containers-must-have-limits
   - Container: app
   - Fix: Add CPU and memory limits

### Suggested Fix:
```yaml
spec:
  containers:
    - name: app
      securityContext:
        runAsNonRoot: true
      resources:
        limits:
          cpu: "500m"
          memory: "256Mi"
```

### Validate Terraform Plan

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: terraform-validator
spec:
  model: google:gemini-2.5-flash
  tools:
    - opa_eval
    - trivy_config_scan

  environment:
    OPA_URL: "${OPA_URL}"

  system_prompt: |
    Validate Terraform plans against security policies.
    Check for common misconfigurations using Trivy.
```

**Usage:**

```bash
# Validate a Terraform plan
terraform plan -out=plan.tfplan
terraform show -json plan.tfplan > plan.json

aofctl run agent terraform-validator.yaml \
  --input "Validate this Terraform plan against our security policies" \
  --file plan.json
```

---

## Code Quality

### Security Code Review

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: security-code-reviewer
spec:
  model: google:gemini-2.5-flash
  tools:
    - sonar_issues_search
    - sonar_hotspots_search

  environment:
    SONAR_URL: "${SONAR_URL}"
    SONAR_TOKEN: "${SONAR_TOKEN}"

  system_prompt: |
    Review code for security issues using SonarQube.
    Focus on OWASP Top 10 and CWE vulnerabilities.
    Provide specific remediation guidance.
```

**Usage:**

```bash
aofctl run agent security-code-reviewer.yaml \
  --input "Review myproject for security vulnerabilities and hotspots"
```

### Quality Gate Check

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: quality-gate
spec:
  model: google:gemini-2.5-flash
  tools:
    - sonar_project_status
    - sonar_measures_component

  environment:
    SONAR_URL: "${SONAR_URL}"
    SONAR_TOKEN: "${SONAR_TOKEN}"

  system_prompt: |
    Check if project passes quality gate.
    Report on failed conditions and metrics.
```

**Usage:**

```bash
aofctl run agent quality-gate.yaml \
  --input "Check if project myapp passes quality gate for branch main"
```

**Expected Output:**

```
## Quality Gate Status: FAILED

### Failed Conditions:
| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| New Coverage | 65.2% | >= 80% | FAIL |
| New Bugs | 3 | 0 | FAIL |

### Current Metrics:
- Coverage: 78.5%
- Bugs: 12
- Vulnerabilities: 3
- Code Smells: 145

### Recommendations:
1. Add tests to increase new code coverage by 15%
2. Fix 3 new bugs introduced in recent commits
```

---

## Complete Security Workflow

### Pre-merge Security Check

```yaml
apiVersion: aof.sh/v1alpha1
kind: Fleet
metadata:
  name: pre-merge-security
spec:
  agents:
    - name: deps
      config:
        tools: [snyk_test]

    - name: code
      config:
        tools: [sonar_issues_search, sonar_hotspots_search]

    - name: infra
      config:
        tools: [trivy_config_scan, opa_eval]

  workflow:
    - step: dependency-scan
      agent: deps
      input: "Scan dependencies for vulnerabilities"

    - step: code-analysis
      agent: code
      input: "Check for security issues in new code"
      parallel: true

    - step: infra-check
      agent: infra
      input: "Validate infrastructure changes"
      parallel: true

    - step: summary
      aggregate: true
      template: |
        ## Pre-merge Security Report

        ### Dependencies: {{ if .steps.dependency-scan.ok }}PASS{{ else }}FAIL{{ end }}
        ### Code: {{ if .steps.code-analysis.ok }}PASS{{ else }}FAIL{{ end }}
        ### Infrastructure: {{ if .steps.infra-check.ok }}PASS{{ else }}FAIL{{ end }}

        ### Overall: {{ if and .steps.dependency-scan.ok .steps.code-analysis.ok .steps.infra-check.ok }}APPROVED{{ else }}BLOCKED{{ end }}
```

**Usage:**

```bash
aofctl run fleet pre-merge-security.yaml \
  --context "PR #123: Add user authentication feature"
```
