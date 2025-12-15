# AOF Examples - Quick Index

## 🎯 Start Here

**New to AOF?** Start with these two files:
1. [`quickstart/hello-world-agent.yaml`](quickstart/hello-world-agent.yaml) - 2 minutes
2. [`quickstart/hello-world-flow.yaml`](quickstart/hello-world-flow.yaml) - 5 minutes

**Full Documentation:**
- 📖 [Complete README](README.md) - Detailed guide with all examples
- ⚡ [Quick Reference](QUICK_REFERENCE.md) - Cheat sheet and common commands  
- 📋 [Manifest](MANIFEST.md) - Statistics and complete inventory

---

## 📂 All Examples by Category

### Quickstart (2)
| File | What It Does | Time |
|------|--------------|------|
| [hello-world-agent.yaml](quickstart/hello-world-agent.yaml) | Simplest agent | 2 min |
| [hello-world-flow.yaml](quickstart/hello-world-flow.yaml) | Webhook → response | 5 min |

### Single Agents (6)
| File | What It Does | Prerequisites |
|------|--------------|---------------|
| [kubernetes-ops-agent.yaml](agents/kubernetes-ops-agent.yaml) | K8s diagnostics & operations | kubectl |
| [github-assistant-agent.yaml](agents/github-assistant-agent.yaml) | PR/issue management | GITHUB_TOKEN |
| [dockerfile-generator-agent.yaml](agents/dockerfile-generator-agent.yaml) | Auto-generate Dockerfiles | - |
| [terraform-planner-agent.yaml](agents/terraform-planner-agent.yaml) | Infra security & cost | terraform |
| [log-analyzer-agent.yaml](agents/log-analyzer-agent.yaml) | Parse & analyze logs | - |
| [security-scanner-agent.yaml](agents/security-scanner-agent.yaml) | Multi-layer security scan | trivy, semgrep |

### Fleets (2)
| File | What It Does | Agents |
|------|--------------|--------|
| [code-review-fleet.yaml](fleets/code-review-fleet.yaml) | Security + Performance + Quality | 3 parallel |
| [sre-oncall-fleet.yaml](fleets/sre-oncall-fleet.yaml) | Diagnose → Fix → Communicate | 3 sequential |

### Flows (6)
| File | What It Does | Trigger |
|------|--------------|---------|
| [slack-qa-bot-flow.yaml](flows/slack-qa-bot-flow.yaml) | Interactive Q&A bot | Slack mention |
| [pr-review-flow.yaml](flows/pr-review-flow.yaml) | Auto-review PRs | GitHub PR |
| [incident-auto-remediation-flow.yaml](flows/incident-auto-remediation-flow.yaml) | Full incident response | PagerDuty |
| [daily-standup-report-flow.yaml](flows/daily-standup-report-flow.yaml) | Daily team status | Cron 9am |
| [deploy-notification-flow.yaml](flows/deploy-notification-flow.yaml) | Track deployments | GitHub deploy |
| [cost-optimization-flow.yaml](flows/cost-optimization-flow.yaml) | Daily cost analysis | Cron 8am |

---

## 🔍 Find by Use Case

**"I want to..."**

| Need | Example File |
|------|--------------|
| Learn AOF basics | [hello-world-agent.yaml](quickstart/hello-world-agent.yaml) |
| Diagnose K8s issues | [kubernetes-ops-agent.yaml](agents/kubernetes-ops-agent.yaml) |
| Automate PR reviews | [pr-review-flow.yaml](flows/pr-review-flow.yaml) |
| Handle incidents | [incident-auto-remediation-flow.yaml](flows/incident-auto-remediation-flow.yaml) |
| Create a Slack bot | [slack-qa-bot-flow.yaml](flows/slack-qa-bot-flow.yaml) |
| Scan for security issues | [security-scanner-agent.yaml](agents/security-scanner-agent.yaml) |
| Optimize cloud costs | [cost-optimization-flow.yaml](flows/cost-optimization-flow.yaml) |
| Review Terraform | [terraform-planner-agent.yaml](agents/terraform-planner-agent.yaml) |
| Analyze logs | [log-analyzer-agent.yaml](agents/log-analyzer-agent.yaml) |
| Generate Dockerfiles | [dockerfile-generator-agent.yaml](agents/dockerfile-generator-agent.yaml) |
| Daily standup reports | [daily-standup-report-flow.yaml](flows/daily-standup-report-flow.yaml) |
| Track deployments | [deploy-notification-flow.yaml](flows/deploy-notification-flow.yaml) |

---

## 📊 By Complexity

### ⭐ Beginner (Start here!)
- [hello-world-agent.yaml](quickstart/hello-world-agent.yaml)
- [hello-world-flow.yaml](quickstart/hello-world-flow.yaml)

### ⭐⭐ Intermediate
- [kubernetes-ops-agent.yaml](agents/kubernetes-ops-agent.yaml)
- [github-assistant-agent.yaml](agents/github-assistant-agent.yaml)
- [dockerfile-generator-agent.yaml](agents/dockerfile-generator-agent.yaml)
- [terraform-planner-agent.yaml](agents/terraform-planner-agent.yaml)
- [log-analyzer-agent.yaml](agents/log-analyzer-agent.yaml)
- [slack-qa-bot-flow.yaml](flows/slack-qa-bot-flow.yaml)
- [daily-standup-report-flow.yaml](flows/daily-standup-report-flow.yaml)
- [deploy-notification-flow.yaml](flows/deploy-notification-flow.yaml)

### ⭐⭐⭐ Advanced
- [security-scanner-agent.yaml](agents/security-scanner-agent.yaml)
- [code-review-fleet.yaml](fleets/code-review-fleet.yaml)
- [sre-oncall-fleet.yaml](fleets/sre-oncall-fleet.yaml)
- [pr-review-flow.yaml](flows/pr-review-flow.yaml)
- [incident-auto-remediation-flow.yaml](flows/incident-auto-remediation-flow.yaml)
- [cost-optimization-flow.yaml](flows/cost-optimization-flow.yaml)

---

## 🚀 Quick Commands

```bash
# Apply any example
aof apply -f <example-file.yaml>

# Query an agent
aof query <agent-name> "<your question>"

# List all examples
ls -R examples/

# View specific example
cat examples/<category>/<file>.yaml
```

---

## 📚 Documentation

- **[README.md](README.md)** - Full documentation (2,500 words)
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - Cheat sheet (1,500 words)
- **[MANIFEST.md](MANIFEST.md)** - Complete inventory & stats

---

**Total Examples**: 16 YAML files (+ 1 legacy CRD)  
**Total Documentation**: 3 comprehensive guides  
**Total Lines of Code**: ~3,500 lines of production-ready YAML

**Ready to start?** Pick an example above and run `aof apply -f` 🚀
