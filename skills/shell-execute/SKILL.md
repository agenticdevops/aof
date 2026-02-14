---
name: shell-execute
description: "Execute shell commands for system operations"
homepage: "https://docs.aof.sh/skills/shell-execute"
metadata:
  emoji: "⚡"
  version: "1.0.0"
  requires:
    bins: ["bash", "sh"]
    env: []
    config: []
  tags: ["shell", "scripting", "operations"]
---

# Shell Execute Skill

Execute shell commands for system operations, diagnostics, and automation.

## When to Use This Skill

- Running system commands
- Automating operational tasks
- Checking system state
- Processing text and data
- Coordinating multiple tools

## Steps

1. **Check environment** — `env | grep KEY`
2. **List files** — `ls -la /path/`
3. **Process text** — `cat file.txt | grep pattern`
4. **Run scripts** — `bash script.sh`
5. **Chain commands** — `cmd1 | cmd2 | cmd3`
