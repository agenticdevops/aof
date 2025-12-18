# Planned AgentFlow Examples

These examples demonstrate **future syntax** (v1alpha1) that is not yet implemented. They showcase the vision for a more declarative DSL with AgentFleet integration.

## Status: Not Yet Implemented

These examples use features that are planned but not yet available:
- `apiVersion: aof.dev/v1alpha1` (planned schema)
- `fleet:` directive (AgentFleet integration)
- `actions:` array (simplified action syntax)
- Trigger types: `github`, `pagerduty`, `cron` with advanced config

## Current Working Syntax

For working examples, see the parent directory which uses `aof.dev/v1` schema:
- `../slack-k8s-bot-flow.yaml` - Slack bot with approval workflow
- `../multi-tenant/` - Multi-tenant routing examples

## Tracking

See [ROADMAP.md](/ROADMAP.md) for implementation status and priorities.

## Contributing

If you'd like to help implement these features, see:
- GitHub Issues: https://github.com/agenticdevops/aof/issues
- Contributing Guide: [CONTRIBUTING.md](/CONTRIBUTING.md)
