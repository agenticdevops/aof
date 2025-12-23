---
sidebar_position: 5
---

# SonarQube Tools

AOF integrates with SonarQube for code quality analysis, security vulnerability detection, and technical debt management.

## Available Tools

| Tool | Description |
|------|-------------|
| `sonar_project_status` | Get quality gate status |
| `sonar_issues_search` | Search for bugs, vulnerabilities, and code smells |
| `sonar_hotspots_search` | Find security hotspots |
| `sonar_measures_component` | Get metrics (coverage, bugs, complexity) |
| `sonar_issue_transition` | Change issue status |
| `sonar_project_analyses` | Get analysis history |

## Configuration

Set the SonarQube server and token:

```bash
export SONAR_URL="https://sonarqube.example.com"
export SONAR_TOKEN="your-sonar-token"
```

## Tool Reference

### sonar_project_status

Get project quality gate status.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | SonarQube server URL |
| `project_key` | string | Yes | Project key in SonarQube |
| `branch` | string | No | Branch name (default: main) |
| `token` | string | Yes | SonarQube token |

**Response:**

```json
{
  "success": true,
  "project_key": "my-project",
  "status": "ERROR",
  "conditions": [
    {
      "metric": "new_coverage",
      "operator": "LT",
      "value": "65.2",
      "threshold": "80",
      "status": "ERROR"
    },
    {
      "metric": "new_security_rating",
      "operator": "GT",
      "value": "1",
      "threshold": "1",
      "status": "OK"
    }
  ]
}
```

### sonar_issues_search

Search for issues in a project.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | SonarQube server URL |
| `project_key` | string | Yes | Project key |
| `types` | string | No | Issue types: `BUG`, `VULNERABILITY`, `CODE_SMELL` |
| `severities` | string | No | Severities: `BLOCKER`, `CRITICAL`, `MAJOR`, `MINOR`, `INFO` |
| `statuses` | string | No | Statuses: `OPEN`, `CONFIRMED`, `RESOLVED`, `CLOSED` |
| `branch` | string | No | Branch name |
| `page` | integer | No | Page number (default: 1) |
| `page_size` | integer | No | Results per page (default: 100) |
| `token` | string | Yes | SonarQube token |

**Response:**

```json
{
  "success": true,
  "total": 42,
  "issues": [
    {
      "key": "AYxyz123",
      "type": "VULNERABILITY",
      "severity": "CRITICAL",
      "message": "Remove this hard-coded password",
      "component": "src/main/java/Auth.java",
      "line": 45,
      "rule": "java:S2068",
      "status": "OPEN",
      "effort": "30min",
      "tags": ["cwe", "owasp-a3"]
    }
  ]
}
```

### sonar_hotspots_search

Search for security hotspots.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | SonarQube server URL |
| `project_key` | string | Yes | Project key |
| `status` | string | No | Hotspot status: `TO_REVIEW`, `REVIEWED` |
| `resolution` | string | No | Resolution: `FIXED`, `SAFE`, `ACKNOWLEDGED` |
| `branch` | string | No | Branch name |
| `token` | string | Yes | SonarQube token |

**Response:**

```json
{
  "success": true,
  "hotspots": [
    {
      "key": "AYabc456",
      "message": "Make sure that using this pseudorandom number generator is safe here",
      "component": "src/main/java/Security.java",
      "line": 23,
      "status": "TO_REVIEW",
      "vulnerability_probability": "MEDIUM",
      "security_category": "weak-cryptography"
    }
  ]
}
```

### sonar_measures_component

Get metrics for a component.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | SonarQube server URL |
| `component` | string | Yes | Component key (project or file) |
| `metrics` | string | Yes | Comma-separated metrics |
| `branch` | string | No | Branch name |
| `token` | string | Yes | SonarQube token |

**Common Metrics:**

- `coverage` - Line coverage %
- `bugs` - Number of bugs
- `vulnerabilities` - Security vulnerabilities
- `code_smells` - Maintainability issues
- `duplicated_lines_density` - Duplication %
- `ncloc` - Lines of code

**Response:**

```json
{
  "success": true,
  "component": "my-project",
  "measures": {
    "coverage": "78.5",
    "bugs": "12",
    "vulnerabilities": "3",
    "code_smells": "145",
    "duplicated_lines_density": "4.2",
    "ncloc": "25000"
  }
}
```

### sonar_issue_transition

Change issue status.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | SonarQube server URL |
| `issue_key` | string | Yes | Issue key |
| `transition` | string | Yes | Transition: `confirm`, `resolve`, `reopen`, `wontfix`, `falsepositive` |
| `comment` | string | No | Comment for the transition |
| `token` | string | Yes | SonarQube token |

### sonar_project_analyses

Get project analysis history.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `endpoint` | string | Yes | SonarQube server URL |
| `project_key` | string | Yes | Project key |
| `branch` | string | No | Branch name |
| `from` | string | No | Start date (YYYY-MM-DD) |
| `to` | string | No | End date (YYYY-MM-DD) |
| `token` | string | Yes | SonarQube token |

## Quality Gate Metrics

| Metric | Description | Typical Threshold |
|--------|-------------|-------------------|
| `coverage` | Line coverage % | >= 80% |
| `new_coverage` | Coverage on new code | >= 80% |
| `bugs` | Total bugs | 0 |
| `vulnerabilities` | Security vulnerabilities | 0 |
| `security_rating` | Security rating (A-E) | A |
| `reliability_rating` | Reliability rating | A |
| `duplicated_lines_density` | Code duplication % | under 3% |

## Example Agent

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: code-quality-auditor
spec:
  model: google:gemini-2.5-flash
  tools:
    - sonar_project_status
    - sonar_issues_search
    - sonar_hotspots_search
    - sonar_measures_component

  environment:
    SONAR_URL: "${SONAR_URL}"
    SONAR_TOKEN: "${SONAR_TOKEN}"

  system_prompt: |
    You are a code quality auditor.

    ## Responsibilities
    - Check quality gate status for projects
    - Identify critical bugs and vulnerabilities
    - Review security hotspots
    - Track code metrics and trends

    ## Prioritization
    1. Security vulnerabilities (CRITICAL, BLOCKER)
    2. Bugs affecting reliability
    3. Security hotspots needing review
    4. Code smells for maintainability
```
