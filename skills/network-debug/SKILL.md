---
name: network-debug
description: "Network troubleshooting and diagnostics"
homepage: "https://docs.aof.sh/skills/network-debug"
metadata:
  emoji: "🌐"
  version: "1.0.0"
  requires:
    bins: ["netstat", "curl"]
    env: []
    config: []
  tags: ["network", "debugging", "connectivity"]
---

# Network Debug Skill

Diagnose network issues and test connectivity.

## When to Use This Skill

- Checking network connectivity
- Debugging DNS resolution
- Inspecting open ports
- Tracing network routes
- Checking network performance

## Steps

1. **Check connectivity** — `curl -v http://endpoint/`
2. **Check DNS** — `nslookup domain.com`
3. **List ports** — `netstat -tulpn`
4. **Test connection** — `nc -zv host port`
5. **Trace route** — `traceroute host`
