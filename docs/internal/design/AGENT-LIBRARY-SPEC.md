# Agent Library Specification - Phase 6 (v1.0.0)

## Overview

Production-ready library of 30+ pre-built agents organized by domain.

## Directory Structure

```
library/
├── kubernetes/          # 5 agents
│   ├── pod-doctor.yaml
│   ├── hpa-tuner.yaml
│   ├── netpol-debugger.yaml
│   ├── yaml-linter.yaml
│   └── resource-optimizer.yaml
├── observability/       # 5 agents
│   ├── alert-manager.yaml
│   ├── slo-guardian.yaml
│   ├── dashboard-generator.yaml
│   ├── log-analyzer.yaml
│   └── trace-investigator.yaml
├── incident/            # 5 agents
│   ├── incident-commander.yaml
│   ├── rca-agent.yaml
│   ├── postmortem-writer.yaml
│   ├── runbook-executor.yaml
│   └── escalation-manager.yaml
├── cicd/               # 5 agents
│   ├── pipeline-doctor.yaml
│   ├── test-analyzer.yaml
│   ├── build-optimizer.yaml
│   ├── release-manager.yaml
│   └── deploy-guardian.yaml
├── security/           # 5 agents (4 exist)
│   ├── security-scanner.yaml     ✅
│   ├── compliance-auditor.yaml   ✅
│   ├── secret-rotator.yaml       ✅
│   ├── vulnerability-patcher.yaml ✅
│   └── threat-hunter.yaml        NEW
├── cloud/              # 5 agents (4 exist)
│   ├── cost-optimizer.yaml       ✅
│   ├── iam-auditor.yaml          ✅
│   ├── resource-rightsize.yaml   ✅
│   ├── cloud-migrator.yaml       ✅
│   └── drift-detector.yaml       NEW
└── README.md
```

## Agent Specifications

### Kubernetes Domain (5 agents)

#### pod-doctor
- **Purpose**: Diagnose pod issues (CrashLoopBackOff, ImagePullBackOff, OOMKilled)
- **Tools**: kubectl
- **Capabilities**:
  - Analyze pod events and logs
  - Identify common failure patterns
  - Recommend fixes

#### hpa-tuner
- **Purpose**: Optimize Horizontal Pod Autoscaler configurations
- **Tools**: kubectl
- **Capabilities**:
  - Analyze HPA metrics and history
  - Recommend scaling thresholds
  - Identify underutilized HPAs

#### netpol-debugger
- **Purpose**: Debug NetworkPolicy issues
- **Tools**: kubectl
- **Capabilities**:
  - Trace network connectivity
  - Analyze policy rules
  - Identify blocking policies

#### yaml-linter
- **Purpose**: Validate and lint Kubernetes YAML manifests
- **Tools**: kubectl (dry-run)
- **Capabilities**:
  - Check schema validity
  - Best practice recommendations
  - Security misconfigurations

#### resource-optimizer
- **Purpose**: Optimize resource requests and limits
- **Tools**: kubectl
- **Capabilities**:
  - Analyze resource utilization
  - Recommend right-sized requests/limits
  - Identify resource waste

### Observability Domain (5 agents)

#### alert-manager
- **Purpose**: Manage and optimize alerting rules
- **Tools**: grafana_*, prometheus tools
- **Capabilities**:
  - Analyze alert fatigue
  - Recommend alert thresholds
  - Deduplicate alerts

#### slo-guardian
- **Purpose**: Monitor SLO compliance and error budgets
- **Tools**: grafana_*, datadog_*
- **Capabilities**:
  - Track SLI metrics
  - Calculate error budget burn rate
  - Alert on SLO breaches

#### dashboard-generator
- **Purpose**: Auto-generate monitoring dashboards
- **Tools**: grafana_dashboard_*
- **Capabilities**:
  - Create dashboards from metrics
  - Template common patterns
  - Golden signals dashboards

#### log-analyzer
- **Purpose**: Analyze logs for patterns and anomalies
- **Tools**: loki_query, aws_logs, gcp_logging
- **Capabilities**:
  - Pattern detection
  - Error correlation
  - Root cause hints

#### trace-investigator
- **Purpose**: Investigate distributed traces
- **Tools**: jaeger tools (future)
- **Capabilities**:
  - Trace analysis
  - Latency breakdown
  - Service dependency mapping

### Incident Domain (5 agents)

#### incident-commander
- **Purpose**: Coordinate incident response
- **Tools**: Multiple (PagerDuty, Slack, etc.)
- **Capabilities**:
  - Declare incident severity
  - Assign roles
  - Track timeline

#### rca-agent
- **Purpose**: Root cause analysis
- **Tools**: kubectl, logs, metrics
- **Capabilities**:
  - Correlate events
  - Timeline reconstruction
  - Impact analysis

#### postmortem-writer
- **Purpose**: Generate incident postmortems
- **Tools**: None (analysis only)
- **Capabilities**:
  - Structure postmortem
  - Extract action items
  - Generate timeline

#### runbook-executor
- **Purpose**: Execute runbook procedures
- **Tools**: kubectl, shell
- **Capabilities**:
  - Step-by-step execution
  - Safety checks
  - Progress tracking

#### escalation-manager
- **Purpose**: Manage escalation chains
- **Tools**: PagerDuty, OpsGenie
- **Capabilities**:
  - Track escalations
  - Notify stakeholders
  - Handoff management

### CI/CD Domain (5 agents)

#### pipeline-doctor
- **Purpose**: Diagnose CI/CD pipeline failures
- **Tools**: github_actions_*, gitlab_ci_*
- **Capabilities**:
  - Analyze failure logs
  - Identify flaky tests
  - Suggest fixes

#### test-analyzer
- **Purpose**: Analyze test results and coverage
- **Tools**: sonar_*, GitHub Actions
- **Capabilities**:
  - Coverage trends
  - Flaky test detection
  - Test performance

#### build-optimizer
- **Purpose**: Optimize build times
- **Tools**: CI/CD tools
- **Capabilities**:
  - Identify bottlenecks
  - Caching recommendations
  - Parallelization opportunities

#### release-manager
- **Purpose**: Coordinate software releases
- **Tools**: GitHub, ArgoCD, Flux
- **Capabilities**:
  - Version management
  - Changelog generation
  - Rollback coordination

#### deploy-guardian
- **Purpose**: Safe deployment validation
- **Tools**: kubectl, ArgoCD
- **Capabilities**:
  - Pre-deployment checks
  - Canary analysis
  - Rollback triggers

### Additional Agents

#### threat-hunter (Security)
- **Purpose**: Proactive threat detection
- **Tools**: Security scanning tools
- **Capabilities**:
  - Anomaly detection
  - Threat indicators
  - Security posture assessment

#### drift-detector (Cloud)
- **Purpose**: Detect infrastructure drift
- **Tools**: terraform, cloud tools
- **Capabilities**:
  - Compare actual vs desired state
  - Drift reporting
  - Remediation recommendations

## Agent Template

```yaml
apiVersion: aof.sh/v1alpha1
kind: Agent
metadata:
  name: {agent-name}
  labels:
    category: {domain}
    domain: {subdomain}
spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192
  temperature: 0.2

  tools:
    - {tool1}
    - {tool2}

  environment:
    # Required environment variables

  system_prompt: |
    You are a {role description}.

    ## Responsibilities
    - {responsibility 1}
    - {responsibility 2}

    ## Process
    1. {step 1}
    2. {step 2}

    ## Output Format
    {structured output}
```

## Implementation Priority

1. **High Priority** (most requested):
   - pod-doctor
   - log-analyzer
   - pipeline-doctor
   - incident-commander
   - rca-agent

2. **Medium Priority**:
   - hpa-tuner
   - alert-manager
   - release-manager
   - postmortem-writer
   - drift-detector

3. **Lower Priority**:
   - netpol-debugger
   - yaml-linter
   - resource-optimizer
   - trace-investigator
   - slo-guardian
   - dashboard-generator
   - test-analyzer
   - build-optimizer
   - deploy-guardian
   - runbook-executor
   - escalation-manager
   - threat-hunter

## Success Criteria

1. All 30+ agents created and documented
2. Each agent has clear system prompt
3. Tools are correctly specified
4. Examples provided for each domain
5. Docusaurus documentation complete
