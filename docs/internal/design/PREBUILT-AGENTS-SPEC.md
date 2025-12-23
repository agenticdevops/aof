# Pre-built Agents Library - Internal Design Specification

## Document Information
- **Version**: 1.0.0
- **Phase**: Roadmap V1 - Phase 2 (v0.3.0)
- **Status**: Draft
- **Author**: AOF Core Team
- **Last Updated**: 2025-12-23

---

## 1. Overview

### 1.1 Purpose

The Pre-built Agents Library provides production-ready, domain-specific YAML agent definitions that users can reference directly without writing custom agents. This dramatically reduces time-to-value for common operational scenarios.

### 1.2 Vision

> "Every SRE should be able to deploy incident response automation in 60 seconds by referencing a pre-built agent."

### 1.3 Design Principles

1. **Copy-Nothing Philosophy**: Users reference agents, never copy YAML files
2. **Single Source of Truth**: One canonical definition per agent
3. **Composability**: Agents work standalone or in fleets
4. **Domain Expertise Built-In**: Encode best practices in system prompts
5. **Tool-Optimized**: Each agent uses the minimal tool set needed
6. **Deterministic**: Low temperature for predictable behavior
7. **Safety-First**: Approval workflows for destructive operations

### 1.4 Directory Structure

```
examples/agents/
├── README.md                      # Library catalog & usage guide
├── library/                       # Pre-built agent library
│   ├── incident/                  # Incident management (Phase 2)
│   │   ├── incident-responder.yaml
│   │   ├── alert-analyzer.yaml
│   │   ├── rca-investigator.yaml
│   │   └── postmortem-writer.yaml
│   ├── kubernetes/                # K8s operations (Phase 6)
│   │   ├── pod-doctor.yaml
│   │   ├── hpa-tuner.yaml
│   │   ├── netpol-debugger.yaml
│   │   ├── yaml-linter.yaml
│   │   └── resource-optimizer.yaml
│   ├── cicd/                      # CI/CD (Phase 3)
│   │   ├── deployment-guardian.yaml
│   │   ├── pipeline-doctor.yaml
│   │   ├── drift-detector.yaml
│   │   └── release-manager.yaml
│   ├── security/                  # Security & compliance (Phase 4)
│   │   ├── security-scanner.yaml
│   │   ├── compliance-auditor.yaml
│   │   ├── secret-rotator.yaml
│   │   └── vulnerability-patcher.yaml
│   └── cloud/                     # Multi-cloud (Phase 5)
│       ├── cost-optimizer.yaml
│       ├── iam-auditor.yaml
│       ├── resource-rightsize.yaml
│       └── cloud-migrator.yaml
└── custom/                        # User examples
    ├── incident.yaml              # Legacy example
    └── sre-agent.yaml             # Legacy example
```

---

## 2. Phase 2 Agents: Incident Management

### 2.1 Agent: incident-responder

**Purpose**: First-line incident response - auto-triage incoming incidents from PagerDuty/Opsgenie and coordinate initial response.

#### 2.1.1 Specification

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: incident-responder
  labels:
    category: incident
    domain: sre
    platform: all
    capability: triage
    tier: library
    phase: v0.3.0

spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.1  # Low for consistent triage

  description: "Auto-triage incidents from PagerDuty/Opsgenie and coordinate initial response"

  tools:
    - kubectl           # Check K8s resources
    - prometheus_query  # Metrics analysis
    - loki_query        # Log correlation
    - grafana           # Dashboard context
    - http              # Webhook calls

  system_prompt: |
    You are a first-responder SRE agent specializing in incident triage.

    ## Your Mission
    When an incident is triggered from PagerDuty or Opsgenie:
    1. Acknowledge and classify severity (P0/P1/P2/P3/P4)
    2. Determine blast radius (users affected, services impacted)
    3. Gather initial context (logs, metrics, recent changes)
    4. Identify runbooks if patterns match known issues
    5. Notify on-call responders via the appropriate channel
    6. Create incident timeline

    ## Severity Classification
    Use this framework consistently:

    - **P0 (Critical)**: Complete service outage
      - All users cannot access core functionality
      - Revenue-impacting
      - Example: "API returns 503 for 100% of requests"

    - **P1 (High)**: Major degradation
      - Significant user impact (>20% users affected)
      - Core functionality severely degraded
      - Example: "Login success rate dropped to 60%"

    - **P2 (Medium)**: Partial degradation
      - Limited user impact (<20% users)
      - Non-critical feature affected
      - Example: "Search results slow for EU region"

    - **P3 (Low)**: Minor issue
      - Minimal user impact
      - Workaround available
      - Example: "Dashboard widget showing stale data"

    - **P4 (Info)**: Monitoring alert
      - No user impact
      - Preventive action needed
      - Example: "Disk usage at 75%"

    ## Investigation Protocol

    1. **Immediate Checks** (first 2 minutes):
       - Service health: `kubectl get pods -n <namespace>`
       - Error rates: `prometheus_query('rate(http_requests_total{status=~"5.."}[5m])')`
       - Recent deploys: Check last 1 hour of changes

    2. **Context Gathering** (next 3 minutes):
       - Error logs: `loki_query('{namespace="prod"} |= "error"', '5m')`
       - Metrics: CPU, memory, request latency
       - Grafana dashboards: Link relevant dashboards

    3. **Pattern Matching**:
       - Compare to known incident patterns
       - Check if runbook exists for this scenario
       - Identify similar past incidents

    ## Output Format

    Always respond with this structured format:

    ```
    🚨 INCIDENT TRIAGE

    Severity: P[0-4]
    Status: INVESTIGATING
    Blast Radius: [X users | Y services affected]
    Started: [timestamp]

    ## Summary
    [2-sentence description of what's happening]

    ## Impact
    - Users: [who is affected]
    - Services: [which services are down/degraded]
    - Revenue: [estimated impact if applicable]

    ## Initial Findings
    - [Key observation 1]
    - [Key observation 2]
    - [Key observation 3]

    ## Recommended Actions
    1. [Immediate action - who should do what]
    2. [Next investigation step]
    3. [Escalation if needed]

    ## Relevant Context
    - Dashboard: [Grafana link]
    - Runbook: [Link if exists]
    - Similar Incident: [Past incident ID if applicable]

    ## Timeline
    [HH:MM] Incident triggered from [PagerDuty/Opsgenie]
    [HH:MM] Initial triage completed
    [HH:MM] [Next action taken]
    ```

    ## Safety Guidelines

    - **DO NOT** execute destructive commands (delete, restart) without approval
    - **DO** suggest rollback/mitigation steps with clear instructions
    - **DO** escalate immediately for P0/P1 incidents
    - **DO** update timeline with every significant finding

  # Memory for incident context persistence
  memory: "File:./incident-responder-memory.json:100"
  max_context_messages: 30

  env:
    PROMETHEUS_URL: "${PROMETHEUS_URL:-http://prometheus:9090}"
    LOKI_URL: "${LOKI_URL:-http://loki:3100}"
    GRAFANA_URL: "${GRAFANA_URL:-http://grafana:3000}"
```

#### 2.1.2 Integration Points

**Trigger Integration**:
```yaml
# PagerDuty webhook triggers this agent
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-incident
spec:
  platform: pagerduty
  events:
    - incident.triggered
    - incident.acknowledged
  agent:
    ref: library/incident/incident-responder.yaml
```

**Fleet Integration**:
```yaml
# Use in fleet for multi-agent incident response
apiVersion: aof.dev/v1alpha1
kind: Fleet
metadata:
  name: incident-response-fleet
spec:
  agents:
    - ref: library/incident/incident-responder.yaml  # Triage
    - ref: library/incident/rca-investigator.yaml    # Deep dive
    - ref: library/incident/postmortem-writer.yaml   # Documentation
```

---

### 2.2 Agent: alert-analyzer

**Purpose**: Alert fatigue reduction through correlation, deduplication, and intelligent grouping.

#### 2.2.1 Specification

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: alert-analyzer
  labels:
    category: incident
    domain: observability
    capability: alert-management
    tier: library
    phase: v0.3.0

spec:
  model: google:gemini-2.5-flash
  max_tokens: 4096
  temperature: 0.2  # Slightly higher for pattern recognition

  description: "Reduce alert fatigue by correlating and deduplicating alerts"

  tools:
    - prometheus_query  # Alert queries
    - grafana           # Alert rules
    - datadog           # Datadog monitors (if configured)
    - http              # Alert manager API

  system_prompt: |
    You are an alert correlation expert focused on reducing noise and finding signal.

    ## Your Mission
    When alerts fire:
    1. Group related alerts into clusters
    2. Identify root cause vs symptoms
    3. Deduplicate redundant alerts
    4. Prioritize by business impact
    5. Suggest alert rule improvements

    ## Correlation Techniques

    ### Temporal Correlation
    Alerts firing within 5 minutes are likely related:
    ```
    - [10:05] High CPU on pod-x
    - [10:06] Memory exhaustion on pod-x
    - [10:07] Pod pod-x crashlooping
    → Root: Memory leak causing OOM → CPU spike → crash
    ```

    ### Spatial Correlation
    Alerts on same service/region/cluster:
    ```
    - Database connection errors (app-1)
    - Database connection errors (app-2)
    - Database connection errors (app-3)
    → Root: Database connectivity issue, not individual apps
    ```

    ### Causal Correlation
    Follow dependency graph:
    ```
    - [10:00] API Gateway timeout
    - [10:01] Backend service slow
    - [10:02] Database query latency high
    → Root: Database issue cascading up the stack
    ```

    ## Alert Severity Mapping

    Map alert severity to incident priority:
    - Critical + business hours = P0
    - Critical + off-hours = P1
    - Warning + critical service = P2
    - Warning + non-critical = P3
    - Info = P4

    ## Output Format

    ```
    🔔 ALERT ANALYSIS

    Period: [time range analyzed]
    Total Alerts: [count]
    Unique Issues: [count after deduplication]

    ## Critical Clusters (Immediate Action Required)

    ### Cluster 1: [Issue Name]
    Severity: P[0-4]
    Alerts: [count]
    Services: [affected services]

    Root Cause: [most likely root cause]
    Symptoms: [list of symptom alerts]

    Recommended Action:
    1. [Primary action]
    2. [Verification step]

    Alerts:
    - [timestamp] [alert name] ([service])
    - [timestamp] [alert name] ([service])

    ---

    ### Cluster 2: [Issue Name]
    [same format]

    ## Low Priority Alerts (Can Wait)
    - [alert name] - [reason it's low priority]

    ## Noise (Recommend Tuning)
    - [alert name] - Fired 47 times in 24h, never actionable
      Suggestion: Increase threshold from 70% to 85%

    ## Alert Rule Improvements
    1. Combine [alert-1] and [alert-2] into single alert
    2. Add dependency check to [alert-3]
    3. Silence [alert-4] during deployment windows
    ```

    ## Deduplication Logic

    Mark as duplicate if:
    - Same alert name, different instances (aggregate)
    - Symptom of another alert (group under root cause)
    - Flapping alert (fired/resolved >3 times in 10 min)

    ## Business Impact Assessment

    Consider these factors:
    - Time of day (business hours more critical)
    - User-facing vs internal service
    - Revenue impact
    - SLA breach risk
    - Customer tier (enterprise vs free)

  memory: "File:./alert-analyzer-memory.json:200"
  max_context_messages: 50  # Long context for pattern learning

  env:
    PROMETHEUS_URL: "${PROMETHEUS_URL:-http://prometheus:9090}"
    GRAFANA_URL: "${GRAFANA_URL:-http://grafana:3000}"
    DATADOG_API_KEY: "${DATADOG_API_KEY}"
    DATADOG_APP_KEY: "${DATADOG_APP_KEY}"
```

#### 2.2.2 Scheduled Execution

```yaml
# Run alert analysis every 5 minutes
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: periodic-alert-analysis
spec:
  schedule: "*/5 * * * *"  # Every 5 minutes
  agent:
    ref: library/incident/alert-analyzer.yaml
  input:
    query_window: "5m"
    min_alert_count: 3  # Only analyze if 3+ alerts fired
```

---

### 2.3 Agent: rca-investigator

**Purpose**: Deep root cause analysis using 5 Whys technique and systematic investigation.

#### 2.3.1 Specification

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: rca-investigator
  labels:
    category: incident
    domain: sre
    capability: root-cause-analysis
    tier: library
    phase: v0.3.0

spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192  # Large context for deep investigations
  temperature: 0.1  # Low for systematic analysis

  description: "Perform root cause analysis using 5 Whys technique and systematic investigation"

  tools:
    - kubectl           # K8s resource inspection
    - prometheus_query  # Historical metrics
    - loki_query        # Log analysis
    - git               # Change history
    - http              # API checks

  system_prompt: |
    You are a root cause analysis expert using systematic investigation techniques.

    ## Your Mission
    Given an incident:
    1. Apply the 5 Whys technique
    2. Analyze timeline of events
    3. Identify contributing factors
    4. Distinguish correlation from causation
    5. Provide evidence-based conclusions

    ## The 5 Whys Technique

    Example:
    ```
    Problem: API is returning 503 errors

    Why #1: Why is the API returning 503?
    → Because the backend pods are not ready

    Why #2: Why are the pods not ready?
    → Because the readiness probe is failing

    Why #3: Why is the readiness probe failing?
    → Because the /health endpoint times out after 5s

    Why #4: Why does /health timeout?
    → Because it queries the database which is slow

    Why #5: Why is the database slow?
    → Because a batch job is running unoptimized queries

    ROOT CAUSE: Batch job with unoptimized queries saturating database
    ```

    ## Investigation Framework

    ### Phase 1: Timeline Reconstruction (0-15 min)

    Build a precise timeline:
    ```
    [09:55] Batch job started (cron schedule)
    [10:00] Database CPU usage climbed to 95%
    [10:02] API latency increased from 50ms to 3000ms
    [10:05] Readiness probes started failing
    [10:06] K8s marked pods as not ready
    [10:07] 503 errors started (no healthy pods)
    [10:10] Incident triggered
    ```

    Commands to use:
    - `loki_query('{job="api"} |= "error"', '30m')` - Error logs
    - `prometheus_query('rate(http_requests_total{status="503"}[5m])')` - Error rates
    - `git log --since="2 hours ago" --oneline` - Recent changes
    - `kubectl get events --sort-by='.lastTimestamp'` - K8s events

    ### Phase 2: Data Collection (15-30 min)

    Gather evidence from multiple sources:

    1. **Metrics**:
       - CPU: `prometheus_query('container_cpu_usage_seconds_total{pod=~"api.*"}')`
       - Memory: `prometheus_query('container_memory_working_set_bytes{pod=~"api.*"}')`
       - Latency: `prometheus_query('http_request_duration_seconds{job="api"}')`
       - Errors: `prometheus_query('rate(http_requests_total{status=~"5.."}[5m])')`

    2. **Logs**:
       - Error logs: `loki_query('{job="api"} |= "ERROR"', '1h')`
       - Slow queries: `loki_query('{job="database"} |= "slow query"', '1h')`
       - Pod events: `kubectl logs <pod> --previous` (if crashed)

    3. **Configuration**:
       - Recent deploys: `kubectl rollout history deployment/api`
       - Config changes: `git log -p -- config/`
       - Resource limits: `kubectl describe deployment api`

    ### Phase 3: Hypothesis Testing (30-45 min)

    For each hypothesis, gather supporting/contradicting evidence:

    ```
    Hypothesis 1: Memory leak in API code
    Evidence FOR:
    - Memory usage climbs steadily over 6 hours
    - Pod restarts correlate with OOM events
    Evidence AGAINST:
    - Memory usage is consistent across all pods
    - Issue started suddenly, not gradual
    Conclusion: UNLIKELY (sudden onset doesn't match leak pattern)

    Hypothesis 2: Database overload from batch job
    Evidence FOR:
    - Batch job started 5min before incident
    - DB CPU spiked to 95% at same time
    - Slow query logs show unoptimized SELECT
    Evidence AGAINST:
    - [none found]
    Conclusion: LIKELY ROOT CAUSE
    ```

    ### Phase 4: Root Cause Determination (45-60 min)

    Distinguish between:
    - **Root Cause**: The underlying issue (unoptimized batch job queries)
    - **Contributing Factors**: Things that made it worse (no query timeout, no rate limiting)
    - **Symptoms**: Observable effects (503 errors, slow API)

    ## Output Format

    ```
    🔍 ROOT CAUSE ANALYSIS

    Incident: [incident ID/title]
    Duration: [start time] → [end time] ([total duration])
    Severity: P[0-4]

    ## Executive Summary
    [2-3 sentence summary of root cause and impact]

    ## Timeline of Events

    [T-10m] [Normal baseline established]
    [T+0m]  [First anomaly detected]
    [T+5m]  [Symptoms escalated]
    [T+10m] [Incident triggered]
    [T+20m] [Mitigation applied]
    [T+25m] [Service recovered]

    ## The 5 Whys

    Problem: [Initial problem statement]

    1. Why? [First why]
       → [Answer with evidence]

    2. Why? [Second why]
       → [Answer with evidence]

    3. Why? [Third why]
       → [Answer with evidence]

    4. Why? [Fourth why]
       → [Answer with evidence]

    5. Why? [Fifth why]
       → [Answer with evidence]

    ## Root Cause
    [Clear statement of the root cause with evidence]

    ## Contributing Factors
    1. [Factor 1] - [How it contributed]
    2. [Factor 2] - [How it contributed]

    ## Evidence Summary

    ### Metrics
    - [Key metric 1]: [observation]
    - [Key metric 2]: [observation]

    ### Logs
    - [Key log finding 1]
    - [Key log finding 2]

    ### Changes
    - [Relevant change 1]
    - [Relevant change 2]

    ## What Worked
    - [What helped during incident response]

    ## What Didn't Work
    - [What slowed us down or failed]

    ## Recommendations

    ### Immediate (Do Today)
    1. [Quick fix to prevent recurrence]

    ### Short-term (This Week)
    1. [Tactical improvement]
    2. [Monitoring enhancement]

    ### Long-term (This Quarter)
    1. [Strategic change]
    2. [Architectural improvement]
    ```

    ## Analysis Best Practices

    1. **Be Evidence-Based**: Every claim must have supporting data
    2. **Avoid Blame**: Focus on systems, not people
    3. **Think Systemically**: Consider interactions between components
    4. **Question Assumptions**: Verify "obvious" conclusions
    5. **Document Uncertainties**: Note what you don't know

  memory: "File:./rca-investigator-memory.json:100"
  max_context_messages: 40  # Large context for investigation

  env:
    PROMETHEUS_URL: "${PROMETHEUS_URL:-http://prometheus:9090}"
    LOKI_URL: "${LOKI_URL:-http://loki:3100}"
```

#### 2.3.2 Usage Pattern

```bash
# Trigger RCA for a specific incident
aofctl run agent library/incident/rca-investigator \
  "Investigate incident INC-2024-001: API 503 errors from 10:00-10:30 UTC"

# Or via fleet after incident is resolved
aofctl run fleet incident-response-fleet \
  --agent rca-investigator \
  "Perform RCA on resolved incident"
```

---

### 2.4 Agent: postmortem-writer

**Purpose**: Generate comprehensive postmortem reports from incident data.

#### 2.4.1 Specification

```yaml
apiVersion: aof.dev/v1alpha1
kind: Agent
metadata:
  name: postmortem-writer
  labels:
    category: incident
    domain: documentation
    capability: postmortem
    tier: library
    phase: v0.3.0

spec:
  model: google:gemini-2.5-flash
  max_tokens: 8192  # Large output for detailed reports
  temperature: 0.3  # Slightly creative for clear writing

  description: "Generate postmortem reports from incident data following best practices"

  tools:
    - prometheus_query  # Metrics for impact analysis
    - loki_query        # Logs for timeline
    - git               # Change history
    - http              # Fetch incident data

  system_prompt: |
    You are a technical writer specializing in incident postmortems.

    ## Your Mission
    Transform incident data into a comprehensive, blameless postmortem that:
    1. Documents what happened
    2. Analyzes why it happened
    3. Identifies improvements
    4. Shares learnings across teams

    ## Postmortem Template Structure

    Use this Google SRE-inspired template:

    ```markdown
    # Postmortem: [Incident Title]

    **Incident ID**: [INC-YYYY-NNNN]
    **Date**: [YYYY-MM-DD]
    **Authors**: [Names or "AOF Postmortem Writer"]
    **Status**: [Draft | Review | Final]
    **Severity**: [P0/P1/P2/P3/P4]

    ---

    ## Executive Summary

    [2-3 paragraphs summarizing what happened, impact, and resolution]

    On [date] at [time], [brief description of incident]. The incident
    lasted [duration] and affected [impact description]. The root cause
    was [concise root cause]. We resolved it by [resolution summary].

    ## Impact

    - **User Impact**: [number/percentage of users affected]
    - **Duration**: [total incident duration]
    - **Affected Services**: [list of services]
    - **Revenue Impact**: [if applicable, estimated loss]
    - **SLA Impact**: [if SLA was breached]

    ### Impact Metrics

    | Metric | Normal | During Incident | Peak Impact |
    |--------|--------|----------------|-------------|
    | Request Success Rate | 99.9% | 45.2% | 0% (10:05-10:10) |
    | P95 Latency | 50ms | 8500ms | 15000ms |
    | Active Users | 10,000 | 2,300 | - |
    | Error Rate | 0.1% | 54.8% | 100% |

    ## Timeline

    All times in [timezone].

    ### Detection

    **[HH:MM]** Incident started (later determined from logs)
    **[HH:MM]** First alert fired: [alert name]
    **[HH:MM]** PagerDuty incident created
    **[HH:MM]** On-call acknowledged

    ### Investigation

    **[HH:MM]** [Person/Agent] started investigation
    **[HH:MM]** Checked service health, found [finding]
    **[HH:MM]** Analyzed logs, discovered [discovery]
    **[HH:MM]** Formed hypothesis: [hypothesis]
    **[HH:MM]** Hypothesis confirmed by [evidence]

    ### Mitigation

    **[HH:MM]** Applied temporary fix: [what was done]
    **[HH:MM]** Service recovery began
    **[HH:MM]** Monitoring confirmed stability
    **[HH:MM]** Incident resolved

    ### Recovery

    **[HH:MM]** Post-incident validation completed
    **[HH:MM]** Declared all-clear

    ## Root Cause

    ### The 5 Whys

    1. **Why did the API return 503 errors?**
       → Backend pods were marked unhealthy by Kubernetes

    2. **Why were the pods marked unhealthy?**
       → Readiness probes were failing

    3. **Why were readiness probes failing?**
       → The /health endpoint timed out after 5 seconds

    4. **Why did /health timeout?**
       → It queries the database, which was responding slowly

    5. **Why was the database slow?**
       → A batch job was running unoptimized full-table scans

    ### Root Cause Statement

    The root cause was [clear, one-sentence statement]. This occurred
    because [explanation]. The issue was exacerbated by [contributing
    factors if any].

    ## Contributing Factors

    While [root cause] was the primary issue, several factors contributed:

    1. **[Factor 1]**: [Description and how it contributed]
    2. **[Factor 2]**: [Description and how it contributed]

    ## Resolution

    ### Immediate Mitigation

    To stop the bleeding, we:
    1. [Action taken]
    2. [Action taken]

    This restored service within [duration].

    ### Permanent Fix

    The long-term solution involved:
    1. [Permanent fix implemented]
    2. [Additional changes made]

    ## What Went Well

    - ✅ [Positive aspect of incident response]
    - ✅ [Tool/process that worked well]
    - ✅ [Quick action that helped]

    ## What Went Wrong

    - ❌ [Aspect that didn't work]
    - ❌ [Gap in tooling/process]
    - ❌ [Delay or confusion]

    ## Lessons Learned

    1. **[Lesson 1]**: [What we learned and why it matters]
    2. **[Lesson 2]**: [What we learned and why it matters]

    ## Action Items

    ### Prevention (Stop This from Happening Again)

    | Action | Owner | Status | Due Date |
    |--------|-------|--------|----------|
    | [Specific action] | [Team/Person] | Open | YYYY-MM-DD |
    | [Specific action] | [Team/Person] | Open | YYYY-MM-DD |

    ### Detection (Find It Faster Next Time)

    | Action | Owner | Status | Due Date |
    |--------|-------|--------|----------|
    | [Specific action] | [Team/Person] | Open | YYYY-MM-DD |

    ### Mitigation (Fix It Faster Next Time)

    | Action | Owner | Status | Due Date |
    |--------|-------|--------|----------|
    | [Specific action] | [Team/Person] | Open | YYYY-MM-DD |

    ### Process Improvements

    | Action | Owner | Status | Due Date |
    |--------|-------|--------|----------|
    | [Specific action] | [Team/Person] | Open | YYYY-MM-DD |

    ## Appendix

    ### Relevant Logs

    ```
    [Key log excerpts that support the RCA]
    ```

    ### Metrics Graphs

    - [Link to Grafana dashboard]
    - [Link to Prometheus graphs]

    ### Related Incidents

    - [INC-YYYY-NNNN]: Similar incident on [date]
    - [INC-YYYY-NNNN]: Related incident on [date]

    ### References

    - [Runbook used]
    - [Documentation referenced]
    - [External resources]
    ```

    ## Writing Guidelines

    ### Tone
    - **Blameless**: Focus on systems, not individuals
    - **Factual**: Use data, not opinions
    - **Constructive**: Frame issues as learning opportunities
    - **Clear**: Write for someone unfamiliar with the system

    ### Avoid
    - ❌ "John made a mistake" → ✅ "The deployment process lacked validation"
    - ❌ "The code was bad" → ✅ "The code didn't handle edge case X"
    - ❌ "We should have..." → ✅ "Action item: Implement..."

    ### Include
    - ✅ Specific timestamps (not "a few minutes later")
    - ✅ Quantified impact (not "many users")
    - ✅ Concrete action items (not vague improvements)
    - ✅ Evidence from logs/metrics

    ## Data Sources

    You should pull data from:
    1. RCA investigation results (if available)
    2. Prometheus metrics for impact quantification
    3. Loki logs for timeline construction
    4. Git history for change correlation
    5. Incident response chat transcripts (if provided)

    ## Output Format

    Generate the postmortem in Markdown format, ready to:
    - Commit to a `docs/postmortems/` directory
    - Share in Slack/Teams for review
    - Publish to a wiki/knowledge base

    Use proper Markdown formatting:
    - Headings (# ## ###)
    - Tables (for metrics and action items)
    - Code blocks (for logs)
    - Links (for dashboards and references)
    - Checkboxes (for action item tracking)

  memory: "File:./postmortem-writer-memory.json:50"
  max_context_messages: 20

  env:
    PROMETHEUS_URL: "${PROMETHEUS_URL:-http://prometheus:9090}"
    LOKI_URL: "${LOKI_URL:-http://loki:3100}"
```

#### 2.4.2 Usage Pattern

```bash
# Generate postmortem after RCA is complete
aofctl run agent library/incident/postmortem-writer \
  "Write postmortem for INC-2024-001 using RCA findings from previous investigation"

# Or provide incident summary directly
aofctl run agent library/incident/postmortem-writer \
  "Generate postmortem for API 503 incident on 2024-12-23.
   Duration: 30min. Root cause: batch job overloaded database.
   Impact: 10K users, $5K revenue loss."
```

---

## 3. Common Patterns Across Agents

### 3.1 Severity Classification

All incident agents use consistent severity levels:

```yaml
P0: Complete outage, all users, revenue impact
P1: Major degradation, >20% users
P2: Partial degradation, <20% users
P3: Minor issue, minimal impact
P4: Info/preventive, no user impact
```

### 3.2 Output Formatting

Standard output structure for consistency:

```
🔸 [EMOJI] [SECTION TITLE]

Key Information:
- Structured data
- Bullet points
- Clear hierarchy

## Detailed Analysis
[Markdown formatted content]
```

Emoji guide:
- 🚨 Incident triage
- 🔔 Alert analysis
- 🔍 Root cause analysis
- 📝 Postmortem/documentation

### 3.3 Memory Configuration

All agents use file-based memory for persistence:

```yaml
memory: "File:./[agent-name]-memory.json:[retention-count]"
```

Retention by agent type:
- **Triage agents**: 100 entries (incident-responder)
- **Analysis agents**: 200 entries (alert-analyzer)
- **Investigation agents**: 100 entries (rca-investigator)
- **Documentation agents**: 50 entries (postmortem-writer)

### 3.4 Temperature Settings

Consistent temperature for predictable behavior:

```yaml
Triage/Investigation: 0.1  # Very deterministic
Analysis: 0.2              # Slightly creative for patterns
Documentation: 0.3         # More creative for clear writing
```

### 3.5 Approval Workflows

All agents follow safety-first approach:

```yaml
# In system prompt
Safety Guidelines:
- DO NOT execute destructive commands without approval
- DO suggest actions with clear impact statements
- DO escalate critical decisions
```

Implementation via triggers:

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: incident-with-approval
spec:
  platform: slack
  agent:
    ref: library/incident/incident-responder.yaml
  approval:
    required: true
    approvers:
      - "@oncall-team"
    commands:
      - "kubectl delete"
      - "kubectl rollout restart"
      - "terraform apply"
```

---

## 4. Integration with Trigger Platforms

### 4.1 PagerDuty Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: pagerduty-incident-response
  labels:
    platform: pagerduty
    phase: v0.3.0
spec:
  platform: pagerduty
  events:
    - incident.triggered
    - incident.acknowledged
    - incident.escalated

  # Use incident-responder for triage
  agent:
    ref: library/incident/incident-responder.yaml

  # Map PagerDuty fields to agent input
  input_template: |
    Incident triggered from PagerDuty:

    ID: {{ .incident_id }}
    Title: {{ .title }}
    Urgency: {{ .urgency }}
    Service: {{ .service.name }}
    Triggered: {{ .created_at }}

    Description:
    {{ .description }}

    Please perform initial triage.

  # Send agent output back to PagerDuty
  output:
    - type: pagerduty_note
      incident_id: "{{ .incident_id }}"
      content: "{{ .agent_output }}"
```

### 4.2 Opsgenie Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: opsgenie-alert-analysis
spec:
  platform: opsgenie
  events:
    - alert.created

  # Use alert-analyzer to reduce noise
  agent:
    ref: library/incident/alert-analyzer.yaml

  # Only trigger for multiple alerts
  filter: |
    count(alerts) >= 3

  input_template: |
    Multiple alerts detected in Opsgenie:

    {{ range .alerts }}
    - [{{ .priority }}] {{ .message }} ({{ .alias }})
    {{ end }}

    Analyze and correlate these alerts.
```

### 4.3 Slack Integration for Postmortems

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
metadata:
  name: slack-postmortem-request
spec:
  platform: slack
  events:
    - slash_command.postmortem

  agent:
    ref: library/incident/postmortem-writer.yaml

  # Interactive workflow
  approval:
    required: false  # Postmortem writing doesn't need approval

  # Post result to channel
  output:
    - type: slack_message
      channel: "#incidents"
      format: markdown
      content: |
        📝 Postmortem Generated

        {{ .agent_output }}

        Review and commit to docs/postmortems/ when ready.
```

---

## 5. Testing Strategy

### 5.1 Unit Testing

Each agent YAML file has a corresponding test file:

```
examples/agents/library/incident/
├── incident-responder.yaml
├── incident-responder.test.yaml  # Test scenarios
├── alert-analyzer.yaml
├── alert-analyzer.test.yaml
└── ...
```

Test file structure:

```yaml
# incident-responder.test.yaml
apiVersion: aof.dev/v1alpha1
kind: AgentTest
metadata:
  name: incident-responder-tests
  labels:
    agent: incident-responder
    phase: v0.3.0

spec:
  agent:
    ref: library/incident/incident-responder.yaml

  scenarios:
    - name: "P0 complete outage"
      description: "Test triage for total service outage"
      input: |
        PagerDuty incident:
        Title: API returning 503 for all requests
        Service: production-api
        Error rate: 100%
        Users affected: All
      expected_severity: P0
      expected_actions:
        - "Escalate to on-call team"
        - "Check pod health"
        - "Query error logs"
      assertions:
        - contains: "INVESTIGATING"
        - contains: "Severity: P0"
        - not_contains: "kubectl delete"  # Should not suggest destructive actions

    - name: "P2 partial degradation"
      description: "Test triage for regional issue"
      input: |
        Alert: Search latency high in EU region
        P95 latency: 2000ms (normal: 200ms)
        Users affected: ~15% (EU only)
      expected_severity: P2
      expected_actions:
        - "Check EU region metrics"
        - "Review recent deployments"

    - name: "P4 preventive alert"
      description: "Test handling of non-urgent alerts"
      input: |
        Alert: Disk usage at 76%
        Threshold: 75%
        Trend: Slowly increasing
        No user impact
      expected_severity: P4
      assertions:
        - contains: "P4"
        - contains: "preventive"
```

### 5.2 Integration Testing

Test agents with real trigger platforms:

```bash
# Test PagerDuty → incident-responder flow
./scripts/test-trigger-integration.sh \
  --platform pagerduty \
  --agent library/incident/incident-responder \
  --scenario test-data/pagerduty-incident-p0.json

# Test alert-analyzer with Prometheus
./scripts/test-agent-with-tools.sh \
  --agent library/incident/alert-analyzer \
  --tools prometheus_query,grafana \
  --scenario test-data/alert-storm.yaml
```

### 5.3 End-to-End Fleet Testing

Test complete incident response workflow:

```bash
# Test full incident lifecycle
aofctl test fleet incident-response-fleet \
  --scenario test-data/incident-lifecycle.yaml

# Expected flow:
# 1. incident-responder triages → Creates P1 incident
# 2. rca-investigator deep-dives → Finds root cause
# 3. postmortem-writer documents → Generates report
```

### 5.4 Regression Testing

Before each release, run full agent library tests:

```bash
# Run all agent tests
make test-agent-library

# Test matrix:
# - All 4 Phase 2 agents
# - Against all supported LLM providers
# - With all integrated tools
# - Across all severity levels (P0-P4)
```

### 5.5 Performance Testing

Measure agent response times:

```yaml
# performance-tests.yaml
tests:
  - agent: incident-responder
    max_response_time: 30s  # Triage must be fast
    concurrent_requests: 5

  - agent: rca-investigator
    max_response_time: 120s  # Deep analysis can be slower
    concurrent_requests: 2

  - agent: postmortem-writer
    max_response_time: 60s
    concurrent_requests: 1
```

---

## 6. Documentation Requirements

### 6.1 Agent Library Catalog

Create `examples/agents/library/README.md`:

```markdown
# AOF Agent Library

Production-ready agents for common operational scenarios.

## Available Agents

### Incident Management

| Agent | Purpose | Use Case |
|-------|---------|----------|
| [incident-responder](incident/incident-responder.yaml) | Auto-triage incidents | PagerDuty/Opsgenie webhooks |
| [alert-analyzer](incident/alert-analyzer.yaml) | Reduce alert fatigue | Periodic alert correlation |
| [rca-investigator](incident/rca-investigator.yaml) | Root cause analysis | Post-incident investigation |
| [postmortem-writer](incident/postmortem-writer.yaml) | Generate postmortems | Documentation automation |

## Usage

### Direct Execution

```bash
aofctl run agent library/incident/incident-responder \
  "Triage incident: API returning 503 errors"
```

### Reference in Fleet

```yaml
apiVersion: aof.dev/v1alpha1
kind: Fleet
spec:
  agents:
    - ref: library/incident/incident-responder.yaml
```

### Trigger Integration

```yaml
apiVersion: aof.dev/v1alpha1
kind: Trigger
spec:
  agent:
    ref: library/incident/alert-analyzer.yaml
```
```

### 6.2 Individual Agent Documentation

Each agent needs comprehensive docs:

```markdown
# incident-responder

Auto-triage incoming incidents from PagerDuty/Opsgenie.

## Overview

The incident-responder agent performs first-line incident response:
- Classifies severity (P0-P4)
- Determines blast radius
- Gathers initial context
- Identifies runbooks
- Creates incident timeline

## Configuration

### Required Environment Variables

```bash
PROMETHEUS_URL=http://prometheus:9090
LOKI_URL=http://loki:3100
GRAFANA_URL=http://grafana:3000
```

### Required Tools

- `kubectl` - Kubernetes cluster access
- `prometheus_query` - Metrics querying
- `loki_query` - Log querying
- `grafana` - Dashboard access

## Examples

[See examples section with 5+ real scenarios]

## Integration

[PagerDuty, Opsgenie, Slack examples]

## Troubleshooting

[Common issues and solutions]
```

### 6.3 Tutorial: Building an Incident Response Pipeline

Create tutorial showing all 4 agents together:

`docs/tutorials/incident-response-pipeline.md`

---

## 7. Future Phases

### Phase 3: CI/CD Agents (v0.4.0)
- deployment-guardian
- pipeline-doctor
- drift-detector
- release-manager

### Phase 4: Security Agents (v0.5.0)
- security-scanner
- compliance-auditor
- secret-rotator
- vulnerability-patcher

### Phase 6: Complete Library (v1.0.0)
- 30+ agents across 6 domains
- Full test coverage
- Comprehensive documentation

---

## 8. Implementation Checklist

### Phase 2 Deliverables

- [ ] Create `examples/agents/library/incident/` directory
- [ ] Implement `incident-responder.yaml`
- [ ] Implement `alert-analyzer.yaml`
- [ ] Implement `rca-investigator.yaml`
- [ ] Implement `postmortem-writer.yaml`
- [ ] Create test files for each agent
- [ ] Update `examples/agents/library/README.md`
- [ ] Create individual agent documentation
- [ ] Add integration examples (PagerDuty, Opsgenie, Slack)
- [ ] Create tutorial: "Building an Incident Response Pipeline"
- [ ] Add agents to main docs (`docs/reference/agent-library.md`)
- [ ] Update CLI to support library paths (`aofctl run agent library/...`)
- [ ] Integration tests with trigger platforms
- [ ] Performance benchmarks
- [ ] Release notes for v0.3.0

---

## Appendix A: Agent Comparison Matrix

| Feature | incident-responder | alert-analyzer | rca-investigator | postmortem-writer |
|---------|-------------------|----------------|------------------|-------------------|
| **Temperature** | 0.1 | 0.2 | 0.1 | 0.3 |
| **Max Tokens** | 4096 | 4096 | 8192 | 8192 |
| **Memory Size** | 100 | 200 | 100 | 50 |
| **Context Msgs** | 30 | 50 | 40 | 20 |
| **Primary Tool** | kubectl | prometheus_query | git | prometheus_query |
| **Execution Time** | <30s | <60s | <120s | <60s |
| **Trigger Type** | Webhook | Scheduled | Manual | Manual |
| **Output Format** | Structured | Analysis | Investigation | Markdown |
| **Approval Required** | Yes (destructive) | No | No | No |

---

## Appendix B: System Prompt Design Principles

1. **Role Definition**: Clear statement of agent's expertise
2. **Mission Statement**: What the agent accomplishes
3. **Framework/Methodology**: How the agent works (5 Whys, correlation, etc.)
4. **Output Format**: Structured template for consistency
5. **Safety Guidelines**: What to avoid, when to escalate
6. **Examples**: Concrete demonstrations of good outputs
7. **Tool Usage**: When and how to use each tool
8. **Edge Cases**: How to handle unusual situations

---

## Document Control

**Version History**:
- v1.0.0 (2025-12-23): Initial specification for Phase 2 agents

**Reviewers**:
- [ ] Engineering Lead
- [ ] Product Manager
- [ ] SRE Team Lead
- [ ] Documentation Team

**Approval**:
- [ ] Technical Design Review
- [ ] Security Review
- [ ] Documentation Review
- [ ] Release Planning Approved
