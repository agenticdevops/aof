---
name: incident-diagnose
description: "Multi-source incident analysis and diagnostics"
homepage: "https://docs.aof.sh/skills/incident-diagnose"
metadata:
  emoji: "🚨"
  version: "1.0.0"
  requires:
    bins: ["kubectl", "curl", "jq"]
    env: []
    config: ["~/.kube/config"]
  tags: ["incident", "diagnostics", "troubleshooting"]
---

# Incident Diagnosis Skill

Systematically diagnose incidents by collecting data from multiple sources (K8s, metrics, logs).

## When to Use This Skill

- Responding to alerts
- Diagnosing service degradation
- Collecting incident context
- Understanding root cause
- Escalating with full context

## Steps

1. **Collect K8s state** — Get pods, events, resources
2. **Check metrics** — Query Prometheus for trends
3. **Review logs** — Search Loki for errors
4. **Correlate data** — Find patterns across sources
5. **Identify root cause** — Match patterns to known issues
6. **Suggest remediation** — Recommend actions
