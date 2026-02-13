---
name: loki-search
description: "Search logs via Loki API"
homepage: "https://docs.aof.sh/skills/loki-search"
metadata:
  emoji: "🔍"
  version: "1.0.0"
  requires:
    bins: ["curl", "jq"]
    env: []
    config: []
  tags: ["logging", "loki", "search"]
---

# Loki Search Skill

Query logs from Loki to find specific events, trace errors, and analyze log patterns.

## When to Use This Skill

- Searching for specific log messages
- Finding errors in large log volumes
- Analyzing log patterns over time
- Correlating logs with metrics
- Debugging multi-service issues

## Steps

1. **Query instant** — `curl 'http://loki:3100/api/prom/query?query=...'`
2. **Query range** — Use start/end for time range queries
3. **Use LogQL** — Write expressions like `{service="api"} | "error"`
4. **Parse results** — Extract timestamps and messages
5. **Analyze patterns** — Look for recurring errors or patterns
