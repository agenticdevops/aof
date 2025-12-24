# Snyk Tool Specification

## 1. Overview

The Snyk Tool provides programmatic access to Snyk's security platform API for vulnerability scanning, dependency analysis, and security monitoring. This tool enables AOF agents to scan projects, track vulnerabilities, and manage security issues across the development lifecycle.

### 1.1 Purpose

- **Dependency Scanning**: Scan project dependencies for known vulnerabilities
- **Container Scanning**: Analyze container images for security issues
- **Code Analysis**: Static application security testing (SAST)
- **Issue Management**: Track and manage security issues
- **License Compliance**: Check dependency licenses

### 1.2 Snyk API Capabilities

Snyk provides comprehensive security platform features:
- **Open Source Security**: Dependency vulnerability detection
- **Container Security**: Image scanning with base image recommendations
- **Code Security**: SAST for common vulnerability patterns
- **IaC Security**: Infrastructure as Code scanning
- **License Compliance**: Open source license tracking

### 1.3 Feature Flag

```toml
[features]
security = ["reqwest", "serde_json"]
```

## 2. Tool Operations

### 2.1 snyk_test

Test a project for vulnerabilities.

**Purpose**: Scan project dependencies and report vulnerabilities.

**Parameters**:
- `path` (required): Project directory path
- `project_type` (optional): Package manager: `npm`, `pip`, `maven`, `gradle`, `go`, `cargo`
- `severity_threshold` (optional): Fail threshold: `low`, `medium`, `high`, `critical`
- `all_projects` (optional): Scan all projects in directory, default: false
- `api_token` (required): Snyk API token

**Response**:
```json
{
  "success": true,
  "ok": false,
  "vulnerabilities_found": 12,
  "summary": {
    "critical": 1,
    "high": 3,
    "medium": 5,
    "low": 3
  },
  "vulnerabilities": [
    {
      "id": "SNYK-JS-LODASH-1018905",
      "title": "Prototype Pollution",
      "package": "lodash",
      "version": "4.17.20",
      "severity": "high",
      "exploit_maturity": "proof-of-concept",
      "fixedIn": ["4.17.21"],
      "upgradePath": ["lodash@4.17.21"]
    }
  ]
}
```

### 2.2 snyk_monitor

Monitor a project for new vulnerabilities.

**Purpose**: Create a snapshot in Snyk for continuous monitoring.

**Parameters**:
- `path` (required): Project directory path
- `project_name` (optional): Custom project name in Snyk
- `org_id` (optional): Snyk organization ID
- `target_reference` (optional): Branch or version reference
- `api_token` (required): Snyk API token

**Response**:
```json
{
  "success": true,
  "project_id": "abc123",
  "project_url": "https://app.snyk.io/org/myorg/project/abc123",
  "issues": {
    "vulnerabilities": {
      "critical": 0,
      "high": 2,
      "medium": 5,
      "low": 8
    }
  }
}
```

### 2.3 snyk_container_test

Scan a container image for vulnerabilities.

**Purpose**: Analyze container images including OS packages and application dependencies.

**Parameters**:
- `image` (required): Image reference (e.g., `nginx:1.25`)
- `dockerfile` (optional): Path to Dockerfile for better analysis
- `platform` (optional): Platform override: `linux/amd64`, `linux/arm64`
- `severity_threshold` (optional): Fail threshold
- `api_token` (required): Snyk API token

**Response**:
```json
{
  "success": true,
  "image": "nginx:1.25",
  "base_image": "debian:bullseye-slim",
  "base_image_recommendation": {
    "current": "debian:bullseye-slim",
    "recommended": "debian:bookworm-slim",
    "vulnerabilities_reduction": 15
  },
  "vulnerabilities": [
    {
      "id": "SNYK-DEBIAN11-OPENSSL-5953844",
      "package": "openssl",
      "version": "1.1.1n-0+deb11u5",
      "severity": "medium",
      "type": "os"
    }
  ]
}
```

### 2.4 snyk_issues_list

List issues for a project or organization.

**Purpose**: Retrieve and filter security issues for reporting and triage.

**Parameters**:
- `org_id` (required): Snyk organization ID
- `project_id` (optional): Filter to specific project
- `severity` (optional): Filter by severity
- `type` (optional): Issue type: `vuln`, `license`
- `ignored` (optional): Include ignored issues, default: false
- `limit` (optional): Max results, default: 100
- `api_token` (required): Snyk API token

**Response**:
```json
{
  "success": true,
  "issues": [
    {
      "id": "issue-123",
      "issue_type": "vuln",
      "severity": "high",
      "title": "Remote Code Execution",
      "package": "log4j-core",
      "version": "2.14.1",
      "project": "my-java-app",
      "introduced_date": "2021-12-10",
      "is_fixable": true,
      "is_patchable": true
    }
  ],
  "total": 45
}
```

### 2.5 snyk_issue_ignore

Ignore a vulnerability with reason.

**Purpose**: Mark false positives or accepted risks.

**Parameters**:
- `org_id` (required): Snyk organization ID
- `project_id` (required): Project ID
- `issue_id` (required): Issue ID to ignore
- `reason` (required): Reason for ignoring: `not-vulnerable`, `wont-fix`, `temporary-ignore`
- `reason_text` (required): Detailed explanation
- `expires` (optional): Expiration date for ignore rule
- `api_token` (required): Snyk API token

**Response**:
```json
{
  "success": true,
  "ignored": true,
  "expires": "2024-03-01T00:00:00Z"
}
```

### 2.6 snyk_fix_pr

Create a fix pull request.

**Purpose**: Generate automated fix PRs for vulnerabilities.

**Parameters**:
- `org_id` (required): Snyk organization ID
- `project_id` (required): Project ID
- `issue_id` (optional): Specific issue to fix
- `api_token` (required): Snyk API token

**Response**:
```json
{
  "success": true,
  "pull_request": {
    "url": "https://github.com/org/repo/pull/123",
    "fixes": [
      {
        "issue_id": "SNYK-JS-LODASH-1018905",
        "from_version": "4.17.20",
        "to_version": "4.17.21"
      }
    ]
  }
}
```

## 3. Authentication

Snyk uses API tokens for authentication:

```bash
export SNYK_TOKEN="your-api-token"
```

Tokens can be scoped to:
- Organization level
- Group level
- Service accounts for CI/CD

## 4. Error Handling

```json
{
  "success": false,
  "error": "authentication failed",
  "error_code": "SNYK_AUTH_ERROR",
  "details": "Invalid or expired API token"
}
```

## 5. Example Agent Usage

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: dependency-security-agent
spec:
  model: google:gemini-2.5-flash
  tools:
    - snyk_test
    - snyk_container_test
    - snyk_issues_list
    - snyk_fix_pr

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"
    SNYK_ORG_ID: "${SNYK_ORG_ID}"

  system_prompt: |
    You are a dependency security analyst.

    ## Capabilities
    - Scan projects for vulnerable dependencies
    - Analyze container images for security issues
    - Recommend and create fix PRs
    - Track vulnerability remediation progress

    ## Workflow
    1. Scan the project/image for vulnerabilities
    2. Prioritize by severity and exploitability
    3. Recommend fixes with upgrade paths
    4. Create fix PRs when requested
```
