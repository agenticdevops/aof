---
name: k8s-debug
description: "Kubernetes pod debugging and troubleshooting"
homepage: "https://docs.aof.sh/skills/k8s-debug"
metadata:
  emoji: "🐳"
  version: "1.0.0"
  requires:
    bins: ["kubectl", "jq"]
    env: []
    config: ["~/.kube/config"]
  tags: ["kubernetes", "debugging", "troubleshooting"]
  author: "AOF Team"
  license: "Apache 2.0"
---

# Kubernetes Debug Skill

Expert guidance for debugging Kubernetes workloads, analyzing pod logs, and troubleshooting common failure patterns.

## When to Use This Skill

- Pod is in CrashLoopBackOff or other error states
- Need to debug application behavior in containers
- Analyzing logs to understand failures
- Investigating pod networking issues
- Checking resource constraints or limits

## Skills & Capabilities

- Retrieve pod status and events from Kubernetes
- Analyze error patterns in logs
- Inspect resource usage and limits
- Suggest fixes based on common issues
- Identify pending pod issues

## Steps

1. **Get pod status** — `kubectl get pod {pod-name} -o wide`
2. **Check events** — `kubectl describe pod {pod-name}`
3. **Retrieve logs** — `kubectl logs {pod-name} --tail=100`
4. **Analyze patterns** — Look for OOMKilled, CrashLoop, ImagePullBackOff
5. **Check resources** — `kubectl top pod {pod-name}`
6. **Diagnose root cause** — Match symptoms to known issues
