# SonarQube Tool Specification

## 1. Overview

The SonarQube Tool provides programmatic access to SonarQube's Web API for code quality analysis, security vulnerability detection, and technical debt management. This tool enables AOF agents to analyze code quality, track issues, and enforce quality gates.

### 1.1 Purpose

- **Code Quality**: Analyze code for bugs, code smells, and maintainability issues
- **Security Analysis**: Detect security vulnerabilities and hotspots
- **Quality Gates**: Enforce quality standards on projects
- **Issue Management**: Track and manage code issues
- **Metrics**: Retrieve coverage, duplication, and complexity metrics

### 1.2 SonarQube API Capabilities

SonarQube provides comprehensive code analysis:
- **Static Analysis**: Detect bugs, vulnerabilities, code smells
- **Security Scanning**: OWASP Top 10, CWE detection
- **Quality Gates**: Pass/fail criteria for projects
- **Coverage Integration**: Test coverage tracking
- **Technical Debt**: Code maintainability tracking

### 1.3 Feature Flag

```toml
[features]
security = ["reqwest", "serde_json"]
```

## 2. Tool Operations

### 2.1 sonar_project_status

Get project quality gate status.

**Purpose**: Check if a project passes quality gates.

**Parameters**:
- `endpoint` (required): SonarQube server URL
- `project_key` (required): Project key in SonarQube
- `branch` (optional): Branch name, default: main branch
- `token` (required): SonarQube token

**Response**:
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

### 2.2 sonar_issues_search

Search for issues in a project.

**Purpose**: Find bugs, vulnerabilities, and code smells.

**Parameters**:
- `endpoint` (required): SonarQube server URL
- `project_key` (required): Project key
- `types` (optional): Issue types: `BUG`, `VULNERABILITY`, `CODE_SMELL`
- `severities` (optional): Severities: `BLOCKER`, `CRITICAL`, `MAJOR`, `MINOR`, `INFO`
- `statuses` (optional): Statuses: `OPEN`, `CONFIRMED`, `RESOLVED`, `CLOSED`
- `branch` (optional): Branch name
- `page` (optional): Page number, default: 1
- `page_size` (optional): Results per page, default: 100
- `token` (required): SonarQube token

**Response**:
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

### 2.3 sonar_hotspots_search

Search for security hotspots.

**Purpose**: Find security-sensitive code that needs review.

**Parameters**:
- `endpoint` (required): SonarQube server URL
- `project_key` (required): Project key
- `status` (optional): Hotspot status: `TO_REVIEW`, `REVIEWED`
- `resolution` (optional): Resolution: `FIXED`, `SAFE`, `ACKNOWLEDGED`
- `branch` (optional): Branch name
- `token` (required): SonarQube token

**Response**:
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

### 2.4 sonar_measures_component

Get metrics for a component.

**Purpose**: Retrieve code metrics like coverage, complexity, duplication.

**Parameters**:
- `endpoint` (required): SonarQube server URL
- `component` (required): Component key (project or file)
- `metrics` (required): Comma-separated metrics: `coverage`, `bugs`, `vulnerabilities`, `code_smells`, `duplicated_lines_density`, `ncloc`
- `branch` (optional): Branch name
- `token` (required): SonarQube token

**Response**:
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

### 2.5 sonar_issue_transition

Change issue status.

**Purpose**: Resolve, confirm, or reopen issues.

**Parameters**:
- `endpoint` (required): SonarQube server URL
- `issue_key` (required): Issue key
- `transition` (required): Transition: `confirm`, `resolve`, `reopen`, `wontfix`, `falsepositive`
- `comment` (optional): Comment for the transition
- `token` (required): SonarQube token

**Response**:
```json
{
  "success": true,
  "issue": {
    "key": "AYxyz123",
    "status": "RESOLVED",
    "resolution": "WONTFIX"
  }
}
```

### 2.6 sonar_project_analyses

Get project analysis history.

**Purpose**: Track analysis trends over time.

**Parameters**:
- `endpoint` (required): SonarQube server URL
- `project_key` (required): Project key
- `branch` (optional): Branch name
- `from` (optional): Start date (YYYY-MM-DD)
- `to` (optional): End date (YYYY-MM-DD)
- `token` (required): SonarQube token

**Response**:
```json
{
  "success": true,
  "analyses": [
    {
      "key": "AY123",
      "date": "2024-01-15T10:30:00+0000",
      "events": [
        {"category": "QUALITY_GATE", "name": "Green (was Red)"}
      ]
    }
  ]
}
```

## 3. Quality Gate Metrics

| Metric | Description | Typical Threshold |
|--------|-------------|-------------------|
| coverage | Line coverage % | >= 80% |
| new_coverage | Coverage on new code | >= 80% |
| bugs | Total bugs | 0 |
| vulnerabilities | Security vulnerabilities | 0 |
| security_rating | Security rating (A-E) | A |
| reliability_rating | Reliability rating | A |
| duplicated_lines_density | Code duplication % | <= 3% |

## 4. Error Handling

```json
{
  "success": false,
  "error": "project not found",
  "error_code": "SONAR_PROJECT_NOT_FOUND",
  "details": "Project 'unknown-project' does not exist"
}
```

## 5. Example Agent Usage

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
