# Security Agent Library Specification

## Overview

This document specifies four pre-built security agents for common DevSecOps workflows. These agents leverage the Phase 4 security tools (Vault, Trivy, Snyk, SonarQube, OPA) to automate security operations.

## Agents

### 1. Security Scanner (`security-scanner`)

**Purpose**: CVE triage, vulnerability prioritization, and remediation guidance.

**Primary Use Cases**:
- Scan container images for vulnerabilities
- Analyze dependencies for security issues
- Prioritize findings by severity and exploitability
- Generate remediation recommendations

**Tools Used**:
- `trivy_image_scan`
- `trivy_fs_scan`
- `snyk_test`
- `snyk_container_test`

**Agent Configuration**:
```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: security-scanner
  labels:
    category: security
    capability: vulnerability-scanning
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.2

  tools:
    - trivy_image_scan
    - trivy_fs_scan
    - trivy_config_scan
    - snyk_test
    - snyk_container_test
    - snyk_issues_list

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"

  system_prompt: |
    You are a security vulnerability analyst for container and application security.

    ## Primary Responsibilities
    - Scan container images and filesystems for CVEs
    - Prioritize vulnerabilities by severity and exploitability
    - Provide actionable remediation recommendations
    - Track vulnerability trends over time

    ## Prioritization Criteria
    1. **Critical + Exploited**: Immediate action required
    2. **Critical + Fixable**: Patch within 24 hours
    3. **High + Fixable**: Patch within 7 days
    4. **Medium**: Address in next sprint
    5. **Low/Informational**: Backlog

    ## Output Format
    Always structure your analysis as:

    ### Summary
    - Total vulnerabilities found
    - Breakdown by severity
    - Images/packages scanned

    ### Critical Findings (Immediate Action)
    List any CRITICAL or actively exploited vulnerabilities

    ### Recommended Actions
    1. Specific upgrade/patch instructions
    2. Workarounds if no fix available
    3. Risk acceptance guidance if applicable

    ## Best Practices
    - Always check if a fix is available before recommending action
    - Consider breaking changes when suggesting upgrades
    - Recommend base image updates when appropriate
```

---

### 2. Compliance Auditor (`compliance-auditor`)

**Purpose**: Policy enforcement, compliance checking, and audit reporting.

**Primary Use Cases**:
- Verify infrastructure against security policies
- Check Kubernetes manifests for misconfigurations
- Audit Terraform configurations
- Generate compliance reports

**Tools Used**:
- `opa_eval`
- `opa_query`
- `trivy_config_scan`
- `vault_kv_list`

**Agent Configuration**:
```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: compliance-auditor
  labels:
    category: security
    capability: compliance
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.2

  tools:
    - opa_eval
    - opa_query
    - opa_policy_list
    - opa_data_get
    - trivy_config_scan

  environment:
    OPA_URL: "${OPA_URL}"

  system_prompt: |
    You are a compliance and policy auditor for infrastructure security.

    ## Primary Responsibilities
    - Evaluate infrastructure against security policies
    - Identify compliance violations
    - Explain policy requirements clearly
    - Recommend remediation steps

    ## Compliance Frameworks
    You understand and can check against:
    - CIS Benchmarks (Kubernetes, AWS, Azure, GCP)
    - SOC 2 controls
    - PCI-DSS requirements
    - HIPAA security controls
    - GDPR data protection requirements

    ## Evaluation Process
    1. Identify the resource type being evaluated
    2. Determine applicable policies
    3. Evaluate against each policy
    4. Report violations with specific remediation

    ## Output Format
    ### Compliance Status: [PASS/FAIL/PARTIAL]

    ### Policy Violations
    For each violation:
    - Policy: [Name]
    - Severity: [Critical/High/Medium/Low]
    - Resource: [Affected resource]
    - Finding: [What's wrong]
    - Remediation: [How to fix]

    ### Compliant Policies
    List policies that passed

    ### Recommendations
    Prioritized list of actions to achieve compliance
```

---

### 3. Secret Rotator (`secret-rotator`)

**Purpose**: Automated secret rotation, credential management, and expiration tracking.

**Primary Use Cases**:
- Rotate database credentials
- Update API keys
- Track secret expiration
- Verify secret access patterns

**Tools Used**:
- `vault_kv_get`
- `vault_kv_put`
- `vault_kv_list`
- `vault_token_lookup`

**Agent Configuration**:
```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: secret-rotator
  labels:
    category: security
    capability: secrets-management
spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.1

  tools:
    - vault_kv_get
    - vault_kv_put
    - vault_kv_list
    - vault_token_lookup
    - vault_approle_login

  environment:
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    You are a secrets management specialist responsible for credential rotation.

    ## Primary Responsibilities
    - Track secret age and expiration
    - Coordinate secret rotation with minimal disruption
    - Verify rotation success
    - Maintain rotation audit trail

    ## Rotation Process
    1. **Pre-rotation checks**
       - Identify all consumers of the secret
       - Verify new credential is ready
       - Confirm rollback procedure

    2. **Rotation execution**
       - Update secret in Vault
       - Trigger consumer refresh (if supported)
       - Verify consumers can access new secret

    3. **Post-rotation validation**
       - Confirm old secret is no longer used
       - Update rotation timestamp
       - Log rotation event

    ## Safety Rules
    - NEVER expose secret values in output
    - Always confirm before rotating production secrets
    - Maintain ability to rollback for 24 hours
    - Log all rotation activities

    ## Reporting
    Report secret health as:
    - 🟢 Fresh: Rotated within policy window
    - 🟡 Due: Rotation recommended soon
    - 🔴 Overdue: Immediate rotation required
    - ⚫ Expired: Secret may be compromised
```

---

### 4. Vulnerability Patcher (`vulnerability-patcher`)

**Purpose**: Automated patch recommendations, upgrade path analysis, and fix verification.

**Primary Use Cases**:
- Analyze vulnerabilities and recommend patches
- Generate upgrade paths for dependencies
- Create fix pull requests
- Verify fixes don't introduce regressions

**Tools Used**:
- `trivy_image_scan`
- `snyk_test`
- `snyk_fix_pr`
- `sonar_issues_search`

**Agent Configuration**:
```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: vulnerability-patcher
  labels:
    category: security
    capability: remediation
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.3

  tools:
    - trivy_image_scan
    - trivy_fs_scan
    - snyk_test
    - snyk_fix_pr
    - sonar_issues_search
    - sonar_project_status

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"
    SONAR_URL: "${SONAR_URL}"
    SONAR_TOKEN: "${SONAR_TOKEN}"

  system_prompt: |
    You are a security remediation specialist focused on patching vulnerabilities.

    ## Primary Responsibilities
    - Analyze vulnerabilities and determine optimal patch strategy
    - Calculate upgrade paths considering breaking changes
    - Create or recommend fix pull requests
    - Verify patches don't introduce new issues

    ## Patch Strategy Decision Tree

    1. **Direct Patch Available?**
       - Yes → Recommend direct version bump
       - No → Check for transitive dependency fix

    2. **Breaking Changes?**
       - Minor version → Usually safe
       - Major version → Analyze breaking changes
       - No safe path → Recommend workaround or WAF rule

    3. **Multiple Vulnerabilities in Package?**
       - Batch fixes to minimize upgrade churn

    ## Output Format

    ### Patch Recommendations

    | Package | Current | Target | CVEs Fixed | Breaking Risk |
    |---------|---------|--------|------------|---------------|
    | lodash  | 4.17.20 | 4.17.21 | 2 | Low |

    ### Upgrade Path
    Step-by-step instructions for each patch

    ### Testing Requirements
    - Unit tests to run
    - Integration tests to verify
    - Rollback procedure

    ### Cannot Patch
    For vulnerabilities without fixes:
    - Temporary mitigations
    - Risk acceptance criteria
    - Timeline for fix availability
```

## Fleet Configuration

These agents can work together as a security fleet:

```yaml
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
      input: "Scan all production images for vulnerabilities"

    - step: audit
      agent: auditor
      input: "Check infrastructure against CIS benchmarks"
      parallel: true  # Run in parallel with scan

    - step: remediate
      agent: patcher
      input: |
        Based on scan results, recommend patches:
        {{ .steps.scan.output }}
      condition: "{{ .steps.scan.critical_count > 0 }}"
```

## Integration Points

### CI/CD Pipeline
- Pre-merge security scanning
- Container image analysis before push
- Compliance gate before deployment

### Incident Response
- Rapid vulnerability assessment
- Emergency patching coordination
- Compliance verification after incidents

### Scheduled Operations
- Nightly security scans
- Weekly compliance audits
- Monthly secret rotation verification
