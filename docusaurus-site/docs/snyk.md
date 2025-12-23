---
sidebar_position: 4
---

# Snyk Tools

AOF integrates with Snyk's security platform for vulnerability scanning, dependency analysis, and automated remediation.

## Available Tools

| Tool | Description |
|------|-------------|
| `snyk_test` | Test a project for vulnerabilities |
| `snyk_monitor` | Monitor a project for new vulnerabilities |
| `snyk_container_test` | Scan container images |
| `snyk_issues_list` | List issues for a project or organization |
| `snyk_issue_ignore` | Ignore a vulnerability with reason |
| `snyk_fix_pr` | Create a fix pull request |

## Configuration

Set the Snyk API token:

```bash
export SNYK_TOKEN="your-snyk-api-token"
```

Get your token from [Snyk Account Settings](https://app.snyk.io/account).

## Tool Reference

### snyk_test

Test a project for vulnerabilities.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Project directory path |
| `project_type` | string | No | Package manager: `npm`, `pip`, `maven`, `gradle`, `go`, `cargo` |
| `severity_threshold` | string | No | Fail threshold: `low`, `medium`, `high`, `critical` |
| `all_projects` | boolean | No | Scan all projects in directory |
| `api_token` | string | Yes | Snyk API token |

**Response:**

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

### snyk_monitor

Monitor a project for new vulnerabilities.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Project directory path |
| `project_name` | string | No | Custom project name in Snyk |
| `org_id` | string | No | Snyk organization ID |
| `target_reference` | string | No | Branch or version reference |
| `api_token` | string | Yes | Snyk API token |

**Response:**

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

### snyk_container_test

Scan a container image for vulnerabilities.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `image` | string | Yes | Image reference (e.g., `nginx:1.25`) |
| `dockerfile` | string | No | Path to Dockerfile for better analysis |
| `platform` | string | No | Platform: `linux/amd64`, `linux/arm64` |
| `severity_threshold` | string | No | Fail threshold |
| `api_token` | string | Yes | Snyk API token |

**Response:**

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

### snyk_issues_list

List issues for a project or organization.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `org_id` | string | Yes | Snyk organization ID |
| `project_id` | string | No | Filter to specific project |
| `severity` | string | No | Filter by severity |
| `type` | string | No | Issue type: `vuln`, `license` |
| `ignored` | boolean | No | Include ignored issues |
| `limit` | integer | No | Max results (default: 100) |
| `api_token` | string | Yes | Snyk API token |

### snyk_issue_ignore

Ignore a vulnerability with reason.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `org_id` | string | Yes | Snyk organization ID |
| `project_id` | string | Yes | Project ID |
| `issue_id` | string | Yes | Issue ID to ignore |
| `reason` | string | Yes | Reason: `not-vulnerable`, `wont-fix`, `temporary-ignore` |
| `reason_text` | string | Yes | Detailed explanation |
| `expires` | string | No | Expiration date for ignore rule |
| `api_token` | string | Yes | Snyk API token |

### snyk_fix_pr

Create a fix pull request.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `org_id` | string | Yes | Snyk organization ID |
| `project_id` | string | Yes | Project ID |
| `issue_id` | string | No | Specific issue to fix |
| `api_token` | string | Yes | Snyk API token |

**Response:**

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

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: dependency-security
spec:
  model: google:gemini-2.5-flash
  tools:
    - snyk_test
    - snyk_container_test
    - snyk_fix_pr

  environment:
    SNYK_TOKEN: "${SNYK_TOKEN}"
    SNYK_ORG_ID: "${SNYK_ORG_ID}"

  system_prompt: |
    You are a dependency security analyst.

    ## Workflow
    1. Scan the project for vulnerabilities
    2. Prioritize by severity and exploitability
    3. Recommend fixes with upgrade paths
    4. Create fix PRs when requested

    ## Focus Areas
    - Critical vulnerabilities with known exploits
    - Dependencies with available patches
    - License compliance issues
```

## Best Practices

1. **Monitor continuously** - Use `snyk_monitor` in CI/CD
2. **Set severity thresholds** - Fail builds on critical issues
3. **Document ignores** - Always provide reason when ignoring
4. **Review PRs carefully** - Automated fixes may have breaking changes
5. **Track base images** - Follow Snyk's base image recommendations
