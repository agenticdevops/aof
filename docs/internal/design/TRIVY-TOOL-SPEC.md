# Trivy Tool Specification

## 1. Overview

The Trivy Tool provides programmatic access to Aqua Security's Trivy scanner for vulnerability detection in containers, filesystems, and IaC configurations. This tool enables AOF agents to scan images, analyze vulnerabilities, and recommend remediation actions.

### 1.1 Purpose

- **Container Scanning**: Scan container images for vulnerabilities
- **Filesystem Scanning**: Scan local directories for vulnerable dependencies
- **IaC Scanning**: Detect misconfigurations in Terraform, Kubernetes manifests
- **SBOM Generation**: Generate Software Bill of Materials
- **CVE Analysis**: Analyze and prioritize vulnerabilities

### 1.2 Trivy Capabilities

Trivy is a comprehensive security scanner supporting:
- **Container Images**: Docker, OCI images
- **Filesystems**: Node.js, Python, Go, Java, Ruby dependencies
- **IaC**: Terraform, CloudFormation, Kubernetes, Dockerfile
- **Kubernetes**: Cluster scanning via operator
- **SBOM**: CycloneDX, SPDX formats

### 1.3 Feature Flag

```toml
[features]
security = ["reqwest", "serde_json"]
```

## 2. Tool Operations

### 2.1 trivy_image_scan

Scan a container image for vulnerabilities.

**Purpose**: Detect CVEs in container images before or after deployment.

**Parameters**:
- `image` (required): Image reference (e.g., `nginx:1.25`, `gcr.io/project/app:v1.0`)
- `severity` (optional): Minimum severity filter: `UNKNOWN`, `LOW`, `MEDIUM`, `HIGH`, `CRITICAL`
- `ignore_unfixed` (optional): Skip vulnerabilities without fixes, default: false
- `format` (optional): Output format: `json`, `table`, `sarif`, default: `json`
- `timeout` (optional): Scan timeout in seconds, default: 300

**Response**:
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
      "title": "HTTP/2 Rapid Reset Attack",
      "description": "..."
    }
  ]
}
```

### 2.2 trivy_fs_scan

Scan a filesystem/directory for vulnerabilities.

**Purpose**: Scan source code or project directories for vulnerable dependencies.

**Parameters**:
- `path` (required): Directory path to scan
- `severity` (optional): Minimum severity filter
- `scanners` (optional): Comma-separated scanners: `vuln`, `secret`, `misconfig`
- `format` (optional): Output format, default: `json`
- `timeout` (optional): Scan timeout in seconds

**Response**:
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

### 2.3 trivy_config_scan

Scan IaC configurations for misconfigurations.

**Purpose**: Detect security misconfigurations in Terraform, K8s manifests, Dockerfiles.

**Parameters**:
- `path` (required): Directory or file path
- `config_type` (optional): Type filter: `terraform`, `kubernetes`, `dockerfile`, `helm`
- `severity` (optional): Minimum severity filter
- `format` (optional): Output format

**Response**:
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

### 2.4 trivy_sbom_generate

Generate Software Bill of Materials.

**Purpose**: Create SBOM for compliance and supply chain security.

**Parameters**:
- `target` (required): Image or directory to scan
- `format` (required): SBOM format: `cyclonedx`, `spdx`, `spdx-json`
- `output` (optional): Output file path

**Response**:
```json
{
  "success": true,
  "format": "cyclonedx",
  "sbom": {
    "bomFormat": "CycloneDX",
    "specVersion": "1.4",
    "components": [...]
  }
}
```

### 2.5 trivy_repo_scan

Scan a remote git repository.

**Purpose**: Scan GitHub/GitLab repos without cloning locally.

**Parameters**:
- `repo` (required): Repository URL (e.g., `https://github.com/org/repo`)
- `branch` (optional): Branch to scan, default: main
- `severity` (optional): Minimum severity filter
- `scanners` (optional): Scanners to use

**Response**:
```json
{
  "success": true,
  "repository": "https://github.com/org/repo",
  "branch": "main",
  "results": [...]
}
```

## 3. Severity Levels

| Severity | Description | Typical Action |
|----------|-------------|----------------|
| CRITICAL | Actively exploited, RCE | Immediate patch |
| HIGH | Significant impact | Patch within 7 days |
| MEDIUM | Moderate impact | Patch within 30 days |
| LOW | Minor impact | Address in next release |
| UNKNOWN | Unclassified | Investigate |

## 4. Integration Patterns

### 4.1 CI/CD Pipeline

```yaml
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

### 4.2 Scheduled Scanning

```yaml
apiVersion: aof.sh/v1alpha1
kind: Trigger
metadata:
  name: nightly-scan
spec:
  source:
    type: schedule
    config:
      cron: "0 2 * * *"  # 2 AM daily

  actions:
    - type: agent
      ref: security-scanner.yaml
      input: "Scan all production images for new vulnerabilities"
```

## 5. Error Handling

```json
{
  "success": false,
  "error": "image not found",
  "error_code": "IMAGE_NOT_FOUND",
  "details": "Failed to pull image: nginx:invalid-tag"
}
```

## 6. Performance Considerations

1. **Caching**: Trivy caches vulnerability DB locally
2. **Parallel Scanning**: Multiple images can be scanned concurrently
3. **Timeouts**: Set appropriate timeouts for large images
4. **Registry Auth**: Configure authentication for private registries
