---
sidebar_position: 5
sidebar_label: CI/CD
---

# CI/CD Agents

Production-ready agents for pipeline management, testing, and release operations.

## Overview

| Agent | Purpose | Tools |
|-------|---------|-------|
| [pipeline-doctor](#pipeline-doctor) | Diagnose pipeline failures | github_*, gitlab_* |
| [test-analyzer](#test-analyzer) | Analyze test results | sonar_*, github_* |
| [build-optimizer](#build-optimizer) | Optimize build times | github_*, gitlab_* |
| [release-manager](#release-manager) | Coordinate releases | git, argocd, flux |
| [deploy-guardian](#deploy-guardian) | Safe deployment validation | kubectl, argocd |

## pipeline-doctor

Diagnoses CI/CD pipeline failures and suggests fixes.

### Usage

```bash
# Diagnose failed pipeline
aofctl run agent library://cicd/pipeline-doctor \
  --prompt "Why did the main branch build fail?"

# Analyze flaky pipelines
aofctl run agent library://cicd/pipeline-doctor \
  --prompt "Which pipelines have the highest failure rate?"
```

### Capabilities

- Analyzes pipeline failure logs
- Identifies flaky tests
- Detects infrastructure issues
- Suggests fixes for common problems
- Tracks failure patterns
- Recommends retry strategies

### Tools Used
- `github_run_list` - List workflow runs
- `github_run_get` - Get run details
- `github_run_logs` - Get run logs
- `gitlab_pipeline_list` - List pipelines
- `gitlab_pipeline_get` - Get pipeline details
- `gitlab_job_log` - Get job logs

### Example Output

```markdown
## Pipeline Diagnosis: main-ci

**Repository**: myorg/api-server
**Workflow**: main-ci.yml
**Run ID**: 12345678
**Status**: FAILED

### Failure Analysis

**Failed Job**: test-integration
**Failed Step**: Run integration tests
**Exit Code**: 1

### Error Summary
```
FAILED tests/integration/test_auth.py::test_oauth_flow
AssertionError: Expected 200, got 503

Connection refused: database:5432
```

### Root Cause
Integration tests failed because the database service container didn't start in time.

### Evidence
1. Database container health check shows "starting" status
2. Test started before database was ready
3. Similar failure in 3 of last 10 runs (30% flaky)

### Recommendations

**Immediate Fix**:
Add wait-for-it script before running tests:
```yaml
- name: Wait for database
  run: |
    ./scripts/wait-for-it.sh database:5432 --timeout=60
```

**Long-term Fix**:
1. Add health check to database service
2. Increase timeout from 30s to 60s
3. Add retry logic for transient failures

### Similar Failures
| Date | Run ID | Same Error | Resolution |
|------|--------|------------|------------|
| Dec 18 | 12345670 | Yes | Auto-retry succeeded |
| Dec 15 | 12345660 | Yes | Manual rerun |
```

---

## test-analyzer

Analyzes test results, coverage, and quality metrics.

### Usage

```bash
# Analyze test coverage
aofctl run agent library://cicd/test-analyzer \
  --prompt "What's the current test coverage for api-server?"

# Find flaky tests
aofctl run agent library://cicd/test-analyzer \
  --prompt "Which tests are most flaky in the last 30 days?"
```

### Capabilities

- Coverage trend analysis
- Flaky test detection
- Test performance metrics
- Quality gate status
- Code smell detection
- Test gap identification

### Tools Used
- `sonar_project_status` - Quality gate status
- `sonar_measures_component` - Code metrics
- `sonar_issues_search` - Find issues
- `github_run_list` - Test run history
- `github_artifacts_list` - Coverage reports

### Example Output

```markdown
## Test Analysis: api-server

### Coverage Summary

| Metric | Current | Target | Trend |
|--------|---------|--------|-------|
| Line Coverage | 78.5% | 80% | -1.2% |
| Branch Coverage | 65.2% | 70% | +0.5% |
| Function Coverage | 82.3% | 80% | PASS |

### Quality Gate: FAILED

**Blocking Issues**:
1. Coverage on new code: 45% (threshold: 80%)
2. 3 new code smells introduced
3. 1 security hotspot unreviewed

### Flaky Tests (Last 30 Days)

| Test | Failures | Pass Rate | Pattern |
|------|----------|-----------|---------|
| test_concurrent_writes | 12 | 88% | Race condition |
| test_oauth_timeout | 8 | 92% | Network flaky |
| test_cache_eviction | 5 | 95% | Timing issue |

### Coverage Gaps

Uncovered critical paths:
1. `src/auth/oauth.rs` - 45% coverage (handles auth)
2. `src/payment/refund.rs` - 52% coverage (handles money)
3. `src/api/rate_limit.rs` - 38% coverage (security)

### Recommendations

1. **Priority 1**: Add tests for payment/refund.rs
   - Missing: refund validation, partial refunds
   - Estimated effort: 4 hours

2. **Priority 2**: Fix flaky test_concurrent_writes
   - Add proper synchronization
   - Use test database per test

3. **Priority 3**: Review security hotspot
   - File: src/auth/jwt.rs:45
   - Issue: Hardcoded secret in test
```

---

## build-optimizer

Optimizes build times and CI/CD resource usage.

### Usage

```bash
# Analyze build performance
aofctl run agent library://cicd/build-optimizer \
  --prompt "Why are our builds taking so long?"

# Get optimization recommendations
aofctl run agent library://cicd/build-optimizer \
  --prompt "How can we reduce build time by 50%?"
```

### Capabilities

- Build time analysis
- Cache utilization review
- Parallelization opportunities
- Resource optimization
- Dependency analysis
- Cost reduction recommendations

### Tools Used
- `github_run_list` - Run history
- `github_run_get` - Run details
- `gitlab_pipeline_list` - Pipeline history
- `gitlab_job_list` - Job details
- `gitlab_job_log` - Build logs

### Example Output

```markdown
## Build Optimization Report: api-server

### Current Performance

| Metric | Value | P90 Industry |
|--------|-------|--------------|
| Average Build Time | 18 min | 8 min |
| Cache Hit Rate | 45% | 85% |
| Parallelization | 2 jobs | 5+ jobs |
| Build Cost/Month | $450 | $200 |

### Time Breakdown

```
Total: 18 minutes
├── Checkout: 30s (3%)
├── Dependencies: 8min (44%) ← BOTTLENECK
├── Compile: 4min (22%)
├── Test: 4min (22%)
└── Deploy: 1.5min (9%)
```

### Optimization Opportunities

**1. Cache Dependencies (Save 6 min)**
Current: Full npm install every build
Fix: Use GitHub Actions cache

```yaml
- uses: actions/cache@v3
  with:
    path: node_modules
    key: ${{ runner.os }}-node-${{ hashFiles('package-lock.json') }}
```
Impact: 8min → 2min for dependencies

**2. Parallelize Tests (Save 2 min)**
Current: Sequential test execution
Fix: Run test suites in parallel

```yaml
jobs:
  test-unit:
    runs-on: ubuntu-latest
  test-integration:
    runs-on: ubuntu-latest
  test-e2e:
    runs-on: ubuntu-latest
```
Impact: 4min → 2min for tests

**3. Use Incremental Compilation**
Current: Full recompile every build
Fix: Enable incremental builds

Impact: 4min → 1.5min for compile

### Projected Results

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Build Time | 18 min | 6 min | 67% faster |
| Cache Hit | 45% | 90% | 2x better |
| Cost/Month | $450 | $150 | 67% savings |
```

---

## release-manager

Coordinates software releases with version management and changelog generation.

### Usage

```bash
# Prepare a release
aofctl run agent library://cicd/release-manager \
  --prompt "Prepare release v2.1.0 for api-server"

# Generate changelog
aofctl run agent library://cicd/release-manager \
  --prompt "Generate changelog since v2.0.0"
```

### Capabilities

- Semantic versioning
- Changelog generation
- Release coordination
- GitOps deployment triggers
- Rollback management
- Release metrics

### Tools Used
- `git` - Version control operations
- `argocd_app_get` - Get app status
- `argocd_app_sync` - Trigger sync
- `argocd_app_list` - List applications
- `flux_kustomization_list` - Flux resources
- `flux_kustomization_get` - Get kustomization
- `flux_reconcile` - Trigger reconcile

### Example Output

```markdown
## Release Preparation: v2.1.0

**Repository**: api-server
**Current Version**: v2.0.3
**Proposed Version**: v2.1.0
**Version Type**: MINOR (new features, backward compatible)

### Changes Since v2.0.0

**Commits**: 47
**Contributors**: 8
**Pull Requests**: 12

### Changelog

## [2.1.0] - 2024-12-20

### Features
- Add OAuth2 PKCE flow support (#123) @alice
- Implement rate limiting per API key (#125) @bob
- Add webhook retry mechanism (#128) @carol

### Bug Fixes
- Fix memory leak in connection pool (#130) @dave
- Resolve race condition in cache (#132) @eve
- Fix timezone handling in reports (#135) @frank

### Performance
- Optimize database queries, 40% faster (#127) @alice
- Add response compression (#129) @bob

### Documentation
- Update API reference for v2 endpoints (#131)
- Add migration guide from v1 (#134)

### Contributors
Thanks to @alice, @bob, @carol, @dave, @eve, @frank, @grace, @henry

### Pre-Release Checklist

- [x] All CI checks passing
- [x] Code coverage > 80%
- [x] No critical security issues
- [x] Changelog generated
- [ ] Release notes reviewed
- [ ] Documentation updated
- [ ] Stakeholders notified

### Deployment Plan

**Stage 1**: Canary (5% traffic)
```bash
argocd app sync api-server --revision v2.1.0 --preview
```

**Stage 2**: Gradual rollout (25% → 50% → 100%)
```bash
flux reconcile kustomization api-server --with-source
```

**Rollback Plan**:
```bash
argocd app rollback api-server
# or
git revert v2.1.0 && git push
```
```

---

## deploy-guardian

Validates deployments for safety and monitors rollouts.

### Usage

```bash
# Validate pre-deployment
aofctl run agent library://cicd/deploy-guardian \
  --prompt "Validate deployment for api-server v2.1.0"

# Monitor canary
aofctl run agent library://cicd/deploy-guardian \
  --prompt "Analyze canary metrics for the current deployment"
```

### Capabilities

- Pre-deployment validation
- Configuration checking
- Canary analysis
- Progressive rollout monitoring
- Automatic rollback triggers
- Health verification

### Tools Used
- `kubectl` - Kubernetes operations
- `argocd_app_get` - App status
- `argocd_app_list` - List apps
- `argocd_app_sync` - Trigger sync
- `argocd_app_history` - Deployment history
- `prometheus_query` - Metrics
- `grafana_query` - Dashboards

### Example Output

```markdown
## Deployment Validation: api-server v2.1.0

### Pre-Deployment Checks

| Check | Status | Details |
|-------|--------|---------|
| Image exists | PASS | gcr.io/myorg/api:v2.1.0 |
| Config valid | PASS | No syntax errors |
| Resources | WARN | Memory limit increased 20% |
| Dependencies | PASS | All services healthy |
| Secrets | PASS | All secrets present |

### Canary Analysis (10% traffic, 15 min)

**Canary**: v2.1.0 (2 pods)
**Stable**: v2.0.3 (18 pods)

| Metric | Canary | Stable | Delta | Status |
|--------|--------|--------|-------|--------|
| Error Rate | 0.12% | 0.10% | +0.02% | PASS |
| P50 Latency | 45ms | 48ms | -6% | PASS |
| P99 Latency | 180ms | 195ms | -8% | PASS |
| Success Rate | 99.88% | 99.90% | -0.02% | PASS |

### Statistical Significance
- Sample size: 15,000 requests per version
- Confidence level: 95%
- P-value: 0.23 (no significant difference)

### Recommendation: PROMOTE

Canary metrics are within acceptable bounds. Recommend proceeding
with progressive rollout.

### Rollout Plan

```
Current: 10% canary
Next:    25% → 50% → 75% → 100%
ETA:     45 minutes total
```

### Rollback Triggers (Automatic)
- Error rate > 1% for 5 minutes
- P99 latency > 500ms for 5 minutes
- Pod crash rate > 10%
- Health check failures > 3
```

---

## Environment Setup

```bash
# GitHub Actions
export GITHUB_TOKEN=ghp_xxxx

# GitLab CI
export GITLAB_TOKEN=glpat-xxxx
export GITLAB_URL=https://gitlab.example.com

# SonarQube
export SONAR_TOKEN=your-token
export SONAR_URL=https://sonarqube.example.com

# ArgoCD
export ARGOCD_SERVER=argocd.example.com
export ARGOCD_AUTH_TOKEN=your-token

# Flux (uses kubeconfig)
export KUBECONFIG=~/.kube/config

# Prometheus/Grafana
export PROMETHEUS_URL=https://prometheus.example.com
export GRAFANA_URL=https://grafana.example.com
export GRAFANA_TOKEN=your-token
```

## Next Steps

- [Security Agents](./security.md) - Security scanning
- [Cloud Agents](./cloud.md) - Cloud operations
- [GitHub Automation Tutorial](../tutorials/github-automation.md)
