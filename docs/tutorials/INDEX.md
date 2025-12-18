# AOF Tutorials

Hands-on tutorials for building automation and bots with AOF.

## Platform Tutorials

### Messaging Bots
- [Telegram Ops Bot](./telegram-ops-bot.md) - Build a Telegram bot for DevOps workflows
  - On-demand runbook execution
  - Incident management
  - Infrastructure status checks
  - Deployment approvals with inline buttons

- [WhatsApp Ops Bot](./whatsapp-ops-bot.md) - Build a WhatsApp bot for on-call workflows
  - Alert acknowledgment and escalation
  - Approval workflows
  - Status checks via chat
  - PagerDuty integration

### CI/CD Automation
- [GitHub Automation](./github-automation.md) - Automated PR reviews and deployments
  - Security and performance code review
  - Auto-labeling and triage
  - Preview environment deployments
  - Release management
  - Multi-repo dependency sync

## Coming Soon

- **Slack Integration** - Advanced Slack bot with approval workflows
- **Discord Bot** - Community management and alerts
- **PagerDuty Integration** - Incident response automation
- **Multi-Platform Routing** - Route messages between platforms
- **Custom Agents** - Build your own agents for automation

## Quick Start

1. **Install AOF**
   ```bash
   curl -sSL https://docs.aof.sh/install.sh | bash
   ```

2. **Pick a platform** - Choose from the tutorials above

3. **Configure & deploy** - Follow the step-by-step guide

## Developer Resources

- [Building Custom Triggers](../developer/BUILDING_TRIGGERS.md) - Create integrations for new platforms
- [AgentFlow Reference](../reference/agentflow.md) - Complete AgentFlow specification
- [API Reference](../reference/api.md) - REST API documentation

## Example Workflows

| Use Case | Platform | Flow |
|----------|----------|------|
| PR Code Review | GitHub | `pr-review-flow` |
| Production Deployment | GitHub → Slack | `production-deploy-flow` |
| On-Call Alert | PagerDuty → WhatsApp | `oncall-alert-flow` |
| Incident Management | Telegram | `incident-manager-flow` |
| Runbook Execution | Telegram/Slack | `runbook-executor-flow` |
| Approval Workflow | WhatsApp | `deployment-approval-flow` |

## Getting Help

- [GitHub Issues](https://github.com/agenticdevops/aof/issues) - Report bugs or request features
- [Documentation](https://docs.aof.sh) - Full documentation
- [Examples](https://github.com/agenticdevops/aof/tree/main/examples) - More example flows
