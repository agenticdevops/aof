---
sidebar_position: 1
---

# Security & Compliance Tools

AOF provides a comprehensive suite of security tools for vulnerability scanning, secrets management, code quality analysis, and policy enforcement.

## Overview

The security tools in AOF enable agents to:

- **Scan for vulnerabilities** in container images, dependencies, and IaC configurations
- **Manage secrets** securely with HashiCorp Vault integration
- **Enforce policies** using Open Policy Agent (OPA)
- **Analyze code quality** and security issues with SonarQube
- **Track and remediate** security findings across your infrastructure

## Available Tools

| Tool | Description | Use Case |
|------|-------------|----------|
| [Vault](/docs/security/vault) | HashiCorp Vault integration | Secrets management, encryption |
| [Trivy](/docs/security/trivy) | Aqua Security Trivy scanner | Container and IaC scanning |
| [Snyk](/docs/security/snyk) | Snyk security platform | Dependency and container security |
| [SonarQube](/docs/security/sonarqube) | Code quality platform | SAST, code smells, bugs |
| [OPA](/docs/security/opa) | Open Policy Agent | Policy enforcement, compliance |

## Pre-built Security Agents

AOF includes four pre-built security agents in the agent library:

| Agent | Purpose |
|-------|---------|
| `security-scanner` | CVE triage, vulnerability prioritization |
| `compliance-auditor` | Policy enforcement, compliance checking |
| `secret-rotator` | Automated secret rotation |
| `vulnerability-patcher` | Patch recommendations, fix PRs |

## Quick Start

### Enable Security Tools

Add the `security` feature to your `Cargo.toml`:

```toml
[dependencies]
aof-tools = { version = "0.2", features = ["security"] }
```

### Use a Pre-built Agent

```bash
# Scan container images for vulnerabilities
aofctl run agent library/security/security-scanner.yaml \
  --input "Scan nginx:1.25 for critical vulnerabilities"

# Check infrastructure compliance
aofctl run agent library/security/compliance-auditor.yaml \
  --input "Audit our Kubernetes manifests against CIS benchmarks"
```

### Create a Custom Security Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: my-security-agent
spec:
  model: google:gemini-2.5-flash
  tools:
    - trivy_image_scan
    - snyk_test
    - vault_kv_get

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"
    VAULT_ADDR: "${VAULT_ADDR}"
    VAULT_TOKEN: "${VAULT_TOKEN}"

  system_prompt: |
    You are a security analyst. Scan for vulnerabilities
    and retrieve secrets securely when needed.
```

## Environment Variables

Configure security tools with these environment variables:

| Variable | Tool | Description |
|----------|------|-------------|
| `VAULT_ADDR` | Vault | Vault server URL |
| `VAULT_TOKEN` | Vault | Authentication token |
| `SNYK_TOKEN` | Snyk | Snyk API token |
| `SONAR_URL` | SonarQube | SonarQube server URL |
| `SONAR_TOKEN` | SonarQube | Authentication token |
| `OPA_URL` | OPA | OPA server URL |

## Security Best Practices

1. **Never hardcode secrets** - Use environment variables or Vault
2. **Rotate credentials regularly** - Use the secret-rotator agent
3. **Scan early and often** - Integrate scanning into CI/CD
4. **Prioritize by severity** - Focus on critical and high vulnerabilities first
5. **Track remediation** - Use the vulnerability-patcher for systematic fixes
