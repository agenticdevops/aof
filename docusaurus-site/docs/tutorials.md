---
sidebar_position: 7
---

# Security Tutorials

Step-by-step tutorials for common security workflows with AOF.

## Tutorial 1: Container Security Pipeline

Build a CI/CD security gate that scans container images before deployment.

### Step 1: Create the Security Gate Agent

```yaml
# security-gate.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: ci-security-gate
  labels:
    category: security
    stage: ci
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.2

  tools:
    - trivy_image_scan
    - snyk_container_test

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"

  system_prompt: |
    You are a CI/CD security gate for container images.

    ## Gate Criteria
    FAIL the build if:
    - Any CRITICAL vulnerabilities with available fixes
    - More than 3 HIGH vulnerabilities
    - Any vulnerability with known exploit in the wild

    WARN but pass if:
    - HIGH vulnerabilities without fixes
    - MEDIUM vulnerabilities

    ## Output Format
    1. Scan Summary
    2. Gate Decision: PASS/FAIL
    3. Critical Findings (if any)
    4. Remediation Steps
```

### Step 2: Run the Security Gate

```bash
# Scan a production image
aofctl run agent security-gate.yaml \
  --input "Scan myapp:v1.2.3 and determine if it passes security gate"

# Scan with specific criteria
aofctl run agent security-gate.yaml \
  --input "Scan nginx:1.25 focusing on CRITICAL and HIGH CVEs only"
```

### Step 3: Integrate with CI/CD

```yaml
# GitHub Actions example
jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - name: Security Gate
        run: |
          aofctl run agent security-gate.yaml \
            --input "Scan ${{ github.repository }}:${{ github.sha }}" \
            --output-format json > scan-result.json

      - name: Check Gate Result
        run: |
          if jq -e '.gate_decision == "FAIL"' scan-result.json; then
            echo "Security gate failed!"
            exit 1
          fi
```

---

## Tutorial 2: Compliance Auditing

Automate compliance checks against CIS benchmarks.

### Step 1: Set Up OPA with Policies

```bash
# Start OPA server
docker run -d -p 8181:8181 openpolicyagent/opa:latest run --server

# Load CIS policies
curl -X PUT http://localhost:8181/v1/policies/cis-k8s \
  -H "Content-Type: text/plain" \
  -d @cis-kubernetes.rego
```

### Step 2: Create the Compliance Agent

```yaml
# compliance-checker.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: compliance-checker
spec:
  model: google:gemini-2.5-flash
  tools:
    - opa_eval
    - opa_query
    - trivy_config_scan

  environment:
    OPA_URL: "http://localhost:8181"

  system_prompt: |
    You are a compliance auditor for Kubernetes and cloud infrastructure.

    ## Compliance Frameworks
    - CIS Kubernetes Benchmark v1.8
    - CIS AWS Foundations Benchmark
    - SOC 2 Type II controls

    ## Audit Process
    1. Identify resource type
    2. Evaluate against all applicable policies
    3. Report violations with severity
    4. Provide specific remediation steps

    ## Output Format
    ### Compliance Status: [PASS/FAIL/PARTIAL]

    #### Violations
    | Control | Severity | Finding | Remediation |
    |---------|----------|---------|-------------|
```

### Step 3: Run Compliance Audit

```bash
# Audit Kubernetes manifests
aofctl run agent compliance-checker.yaml \
  --input "Audit the deployment.yaml against CIS Kubernetes benchmarks" \
  --file deployment.yaml

# Audit Terraform configurations
aofctl run agent compliance-checker.yaml \
  --input "Check terraform/ directory against CIS AWS benchmarks"
```

---

## Tutorial 3: Secret Rotation Workflow

Automate credential rotation with Vault integration.

### Step 1: Set Up Vault

```bash
# Start Vault dev server
vault server -dev

# Enable KV secrets engine
vault secrets enable -path=secret kv-v2

# Store initial secrets
vault kv put secret/myapp/db \
  username=dbadmin \
  password=initial-password \
  rotated_at="2024-01-01T00:00:00Z"
```

### Step 2: Create the Rotation Agent

```yaml
# secret-rotator.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: secret-rotator
spec:
  model: google:gemini-2.5-flash
  temperature: 0.1
  tools:
    - vault_kv_get
    - vault_kv_put
    - vault_kv_list
    - vault_token_lookup

  environment:
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    You are a secrets rotation specialist.

    ## Rotation Policy
    - Database credentials: Rotate every 30 days
    - API keys: Rotate every 90 days
    - Service accounts: Rotate every 180 days

    ## Safety Rules
    - NEVER display secret values in output
    - Always verify token permissions before rotation
    - Log rotation events with timestamps
    - Confirm rotation success before declaring complete

    ## Rotation Status
    - Fresh: Less than 50% of rotation period
    - Due: 50-100% of rotation period
    - Overdue: Past rotation period
    - Expired: Way past rotation (potential breach)
```

### Step 3: Check and Rotate Secrets

```bash
# Check secret rotation status
aofctl run agent secret-rotator.yaml \
  --input "Check rotation status for all secrets in secret/myapp/"

# Rotate a specific secret
aofctl run agent secret-rotator.yaml \
  --input "Rotate the database credentials at secret/myapp/db"
```

---

## Tutorial 4: Vulnerability Remediation

Automate patch recommendations and fix PRs.

### Step 1: Create the Patcher Agent

```yaml
# vulnerability-patcher.yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: vulnerability-patcher
spec:
  model: google:gemini-2.5-flash
  tools:
    - trivy_fs_scan
    - snyk_test
    - snyk_fix_pr
    - sonar_issues_search

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"
    SNYK_ORG_ID: "${SNYK_ORG_ID}"

  system_prompt: |
    You are a vulnerability remediation specialist.

    ## Patch Strategy
    1. Scan for vulnerabilities
    2. Prioritize by severity and exploitability
    3. Check if fixes are available
    4. Recommend upgrade paths
    5. Create fix PRs when safe

    ## Breaking Change Analysis
    - Minor version bump: Usually safe
    - Major version bump: Review changelog
    - No fix available: Recommend workarounds

    ## Output Format
    | Package | Current | Target | CVEs Fixed | Risk |
    |---------|---------|--------|------------|------|
```

### Step 2: Scan and Remediate

```bash
# Scan and get remediation plan
aofctl run agent vulnerability-patcher.yaml \
  --input "Scan ./myproject and create a remediation plan for all HIGH+ vulnerabilities"

# Create fix PRs
aofctl run agent vulnerability-patcher.yaml \
  --input "Create fix PRs for all critical vulnerabilities in project abc123"
```

---

## Tutorial 5: Security Fleet Orchestration

Coordinate multiple security agents for comprehensive scanning.

### Step 1: Create Fleet Configuration

```yaml
# security-fleet.yaml
apiVersion: aof.sh/v1alpha1
kind: Fleet
metadata:
  name: security-operations
spec:
  agents:
    - name: scanner
      ref: library/security/security-scanner.yaml

    - name: auditor
      ref: library/security/compliance-auditor.yaml

    - name: patcher
      ref: library/security/vulnerability-patcher.yaml

  workflow:
    - step: scan
      agent: scanner
      input: "Scan all production container images"

    - step: audit
      agent: auditor
      input: "Check Kubernetes configs against CIS benchmarks"
      parallel: true  # Run in parallel with scan

    - step: remediate
      agent: patcher
      input: |
        Create remediation plan based on findings:
        Scan: {{ .steps.scan.output }}
        Audit: {{ .steps.audit.output }}
      condition: "{{ .steps.scan.critical_count > 0 }}"
```

### Step 2: Run the Fleet

```bash
# Execute complete security workflow
aofctl run fleet security-fleet.yaml

# Run specific steps
aofctl run fleet security-fleet.yaml --step scan,audit
```

---

## Next Steps

- [Security Tool Reference](/docs/security/overview)
- [Pre-built Security Agents](/docs/agents/library)
- [Fleet Orchestration](/docs/fleets/overview)
