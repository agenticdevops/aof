# Conversational Squad Building

Build coordinated agent teams through natural language conversations.

## What Is a Squad?

A squad is a team of specialized agents that work together on a common goal. Each agent has:
- A specific role and expertise
- Complementary skills
- Clear responsibilities and boundaries
- Coordination patterns for collaboration

## Available Squads

### Incident Response Squad

**Use case:** Rapid incident triage, investigation, and remediation

**Agents:**
- **Triage Specialist** (🚨): Classifies severity, gathers context, creates timeline
- **Log Detective** (🔍): Searches logs, identifies patterns, constructs forensics
- **Performance Inspector** (📊): Queries metrics, correlates data, detects anomalies
- **Action Commander** (⚡): Executes safe remediation, runs runbooks, performs rollbacks

**Coordination:** Hierarchical - triage leads, broadcasts critical findings

### Monitoring Squad

**Use case:** Proactive health monitoring and alerting for clusters and applications

**Agents:**
- **Cluster Guardian** (🛡️): Monitors pod/node health, checks events, verifies resources
- **Performance Sentinel** (📈): Queries metrics, checks SLO compliance, generates alerts
- **Notification Hub** (📢): Posts alerts, routes by severity, escalates to on-call

**Coordination:** Parallel monitoring with centralized alerting

### Deployment Squad

**Use case:** Safe deployment automation with pre-flight checks and verification

**Agents:**
- **Launch Controller** (✅): Validates cluster health, checks dependencies, confirms resources
- **Deployment Executor** (🚀): Applies manifests, executes Helm deploys, performs rolling updates
- **Quality Inspector** (🔬): Monitors rollout, runs smoke tests, verifies health

**Coordination:** Sequential pipeline - pre-flight → deploy → verify

### Cost Optimization Squad

**Use case:** Cloud cost analysis and savings implementation

**Agents:**
- **Budget Guardian** (💰): Parses billing data, identifies trends, categorizes spend
- **Efficiency Advisor** (💡): Suggests right-sizing, recommends reserved instances, identifies idle resources
- **Savings Executor** (🔧): Modifies Terraform, resizes instances, removes waste

**Coordination:** Pipeline - analyze → recommend → implement

## Building a Squad

### Basic Squad Creation

```
You: Build me an incident response squad
```

The system will generate:
1. 4 coordinated agents with incident response expertise
2. AGENTS.md entries with capabilities and boundaries
3. SOUL.md personalities with communication styles
4. squads.yaml configuration for coordination

### Customizing for Your Domain

```
You: Build me an incident response squad for Postgres
```

Domain customization adapts:
- Agent capabilities to mention Postgres expertise
- Personalities to reflect database knowledge
- Default introductions to include domain context

Example: Triage Specialist becomes "Triage Specialist specialized in incident response for Postgres"

### Listing Available Squads

```
You: What squads can you build?
```

Response lists all 4 templates with descriptions.

## What Gets Generated

### AGENTS.md Entries

Each agent gets a YAML entry with complete specifications:

```yaml
- id: incident-triage
  name: Triage Specialist
  role: First Responder
  avatar: 🚨
  personality_traits:
    - calm under pressure
    - methodical
    - decisive
  skills:
    - alert-parsing
    - severity-classification
  can:
    - Classify incident severity (P0-P4)
    - Parse and correlate alert data
  cannot:
    - Execute remediation without approval
    - Make architectural decisions during incident
```

### SOUL.md Personalities

Each agent gets a personality section:

```markdown
## incident-triage

\```yaml
id: incident-triage
communication_style: professional
tone: calm-professional
personality_summary: First Responder for incident triage
values:
  - accuracy
  - efficiency
boundaries:
  - Never execute remediation without approval
default_intro: I'm Triage Specialist, your First Responder. I classify incident severity.
\```

# Communication Style

I focus on rapid assessment and clear communication during high-pressure situations.
```

### squads.yaml Configuration

Coordination rules for the squad:

```yaml
squads:
  - name: incident-response
    agents:
      - incident-triage
      - log-analyzer
      - metric-checker
      - remediation-executor
    coordination:
      type: hierarchical - triage leads
      communication: broadcast to all on critical findings
```

## Examples

### Example 1: Build Monitoring Squad

```
You: I need a monitoring squad for my Kubernetes cluster

System: Created monitoring squad with 3 agents.

Agents: Cluster Guardian, Performance Sentinel, Notification Hub

Review the generated AGENTS.md, SOUL.md, and squads.yaml files.
```

### Example 2: Build Deployment Squad with Domain

```
You: Build deployment squad for our microservices

System: Created deployment squad with 3 agents, customized for microservices domain.

Agents: Launch Controller, Deployment Executor, Quality Inspector

Review the generated AGENTS.md, SOUL.md, and squads.yaml files.
```

### Example 3: Unknown Squad Type

```
You: Build me a database optimization squad

System: I couldn't determine which squad template to use. Available squads:

- **incident-response**: Rapid incident triage, investigation, and remediation
- **monitoring**: Proactive health monitoring and alerting
- **deployment**: Safe deployment automation with validation
- **cost-optimization**: Cloud cost analysis and optimization

Which would you like?
```

## Customization Hints

After generating a squad, each template includes hints for further customization:

**Incident Response:**
- Add domain-specific log parsers
- Customize severity classification for your SLOs
- Add runbook skills specific to your infrastructure

**Monitoring:**
- Add application-specific health checks
- Configure alert routing for your team structure
- Customize SLO thresholds

**Deployment:**
- Add custom health checks for your apps
- Configure deployment strategies (blue/green, canary)
- Add smoke test suites

**Cost Optimization:**
- Configure cloud provider-specific cost APIs
- Add company-specific cost allocation tags
- Customize optimization strategies

## Next Steps

1. Review generated files in your workspace
2. Customize agent capabilities for your environment
3. Add domain-specific skills (use "Learn how to..." for skill teaching)
4. Test the squad with a trial scenario
5. Refine based on real-world usage

