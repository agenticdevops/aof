# Agent Flows

Flows define **context-agnostic orchestration logic**. They work with any trigger (Slack, Telegram, PagerDuty) and any context (prod, staging, dev).

## Available Flows

- **`k8s-ops-flow.yaml`** - Kubernetes operations workflow
  - Detects: kubectl, k8s, kubernetes, pod, deploy, scale
  - Agent: k8s-ops
  - Features: Approval workflow for destructive ops

- **`incident-flow.yaml`** - Incident response workflow
  - Detects: incident, outage, alert, critical, p0, p1
  - Fleet: incident-team
  - Features: Auto-investigation, severity classification

- **`deploy-flow.yaml`** - Deployment workflow
  - Detects: deploy, release, rollout, rollback
  - Agents: security, k8s-ops, devops
  - Features: Security scan, validation, approval

## Design Pattern

Flows are **platform and environment agnostic**:

\`\`\`yaml
# ❌ BAD: Hardcoded Slack and prod cluster
trigger:
  type: Slack
  config:
    bot_token: xoxb-prod-token
    kubeconfig: /prod/kubeconfig

# ✅ GOOD: References via bindings
trigger:
  patterns: ["kubectl", "k8s"]
# Binding provides: trigger + context
\`\`\`

## Usage

Flows are used via **bindings**:

\`\`\`yaml
# bindings/prod-slack-k8s.yaml
spec:
  trigger: triggers/slack-prod.yaml
  flow: flows/k8s-ops-flow.yaml
  context: contexts/prod.yaml
\`\`\`

Same flow, different environments:
- `prod-slack-k8s` → Production cluster, strict approvals
- `staging-slack-k8s` → Staging cluster, relaxed approvals
- `dev-slack-k8s` → Local cluster, no approvals

## Best Practices

1. **Keep context-agnostic** - No hardcoded credentials or kubeconfigs
2. **Use pattern matching** - Trigger on keywords, not channels
3. **Implement approvals** - Check for `requires_approval: true`
4. **Handle errors** - Add error handling nodes
5. **Document clearly** - Explain the workflow logic
