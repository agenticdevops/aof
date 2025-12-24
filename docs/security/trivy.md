---
sidebar_position: 3
---

# Trivy Tools

AOF integrates with Aqua Security's Trivy scanner for comprehensive vulnerability detection in containers, filesystems, and IaC configurations.

## Available Tools

| Tool | Description |
|------|-------------|
| `trivy_image_scan` | Scan container images for vulnerabilities |
| `trivy_fs_scan` | Scan filesystems for vulnerable dependencies |
| `trivy_config_scan` | Scan IaC configurations for misconfigurations |
| `trivy_sbom_generate` | Generate Software Bill of Materials |
| `trivy_repo_scan` | Scan remote git repositories |

## Prerequisites

Install Trivy CLI:

```bash
# macOS
brew install trivy

# Linux
curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin

# Docker
docker pull aquasec/trivy
```

## Tool Reference

### trivy_image_scan

Scan a container image for vulnerabilities.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `image` | string | Yes | Image reference (e.g., `nginx:1.25`) |
| `severity` | string | No | Minimum severity: `UNKNOWN`, `LOW`, `MEDIUM`, `HIGH`, `CRITICAL` |
| `ignore_unfixed` | boolean | No | Skip vulnerabilities without fixes |
| `format` | string | No | Output format: `json`, `table`, `sarif` |
| `timeout` | integer | No | Scan timeout in seconds (default: 300) |

**Example:**

```yaml
tools:
  - trivy_image_scan

# "Scan nginx:1.25 for HIGH and CRITICAL vulnerabilities"
```

**Response:**

```json
{
  "success": true,
  "image": "nginx:1.25",
  "summary": {
    "total": 45,
    "critical": 2,
    "high": 8,
    "medium": 20,
    "low": 15
  },
  "vulnerabilities": [
    {
      "id": "CVE-2023-44487",
      "package": "nghttp2",
      "installed_version": "1.51.0",
      "fixed_version": "1.57.0",
      "severity": "HIGH",
      "title": "HTTP/2 Rapid Reset Attack"
    }
  ]
}
```

### trivy_fs_scan

Scan a filesystem or directory for vulnerable dependencies.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Directory path to scan |
| `severity` | string | No | Minimum severity filter |
| `scanners` | string | No | Comma-separated scanners: `vuln`, `secret`, `misconfig` |
| `format` | string | No | Output format |
| `timeout` | integer | No | Scan timeout in seconds |

**Example:**

```yaml
# "Scan /app for vulnerabilities and exposed secrets"
```

**Response:**

```json
{
  "success": true,
  "path": "/app",
  "results": [
    {
      "target": "package.json",
      "type": "npm",
      "vulnerabilities": [
        {
          "id": "CVE-2024-12345",
          "package": "lodash",
          "installed": "4.17.20",
          "fixed": "4.17.21",
          "severity": "HIGH"
        }
      ]
    }
  ]
}
```

### trivy_config_scan

Scan IaC configurations for misconfigurations.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Directory or file path |
| `config_type` | string | No | Type: `terraform`, `kubernetes`, `dockerfile`, `helm` |
| `severity` | string | No | Minimum severity filter |
| `format` | string | No | Output format |

**Example:**

```yaml
# "Check our Terraform configs for security misconfigurations"
```

**Response:**

```json
{
  "success": true,
  "path": "/terraform",
  "misconfigurations": [
    {
      "id": "AVD-AWS-0057",
      "title": "S3 bucket has public access enabled",
      "severity": "HIGH",
      "file": "s3.tf",
      "line": 15,
      "resolution": "Set 'block_public_acls' to true"
    }
  ]
}
```

### trivy_sbom_generate

Generate a Software Bill of Materials (SBOM).

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `target` | string | Yes | Image or directory to scan |
| `format` | string | Yes | SBOM format: `cyclonedx`, `spdx`, `spdx-json` |
| `output` | string | No | Output file path |

**Example:**

```yaml
# "Generate a CycloneDX SBOM for our production image"
```

### trivy_repo_scan

Scan a remote git repository.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `repo` | string | Yes | Repository URL |
| `branch` | string | No | Branch to scan (default: main) |
| `severity` | string | No | Minimum severity filter |
| `scanners` | string | No | Scanners to use |

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: container-scanner
spec:
  model: google:gemini-2.5-flash
  tools:
    - trivy_image_scan
    - trivy_config_scan

  system_prompt: |
    You are a container security specialist.

    When scanning images:
    1. Focus on CRITICAL and HIGH vulnerabilities
    2. Identify if fixes are available
    3. Recommend base image upgrades when applicable
    4. Check for misconfigurations in Dockerfiles
```

## Severity Levels

| Severity | Description | Recommended Action |
|----------|-------------|-------------------|
| CRITICAL | Actively exploited, RCE | Immediate patch |
| HIGH | Significant impact | Patch within 7 days |
| MEDIUM | Moderate impact | Patch within 30 days |
| LOW | Minor impact | Address in next release |
| UNKNOWN | Unclassified | Investigate |

## CI/CD Integration

```yaml
# Example: Security gate in CI
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: ci-security-gate
spec:
  model: google:gemini-2.5-flash
  tools:
    - trivy_image_scan
    - trivy_config_scan

  system_prompt: |
    You are a CI/CD security gate.

    Scan the provided image and fail the build if:
    - Any CRITICAL vulnerabilities exist
    - More than 5 HIGH vulnerabilities exist

    Provide clear remediation guidance for each finding.
```
