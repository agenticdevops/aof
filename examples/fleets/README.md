# Fleet Compositions

Fleets are **compositions of multiple agents** working together on complex tasks. Define fleets once here and reference them in flows or bindings.

## Available Fleets

- **`code-review-team.yaml`** - Security + K8s Ops
  - Security scanner reviews code and containers
  - K8s ops validates deployment manifests
  - **Use for**: PR reviews, deployment validation

- **`incident-team.yaml`** - Incident + K8s Ops + Security
  - Incident coordinator leads response
  - K8s ops provides diagnostics
  - Security checks for breaches
  - **Use for**: Production incidents, post-mortems

## Usage

### Reference in Flows
\`\`\`yaml
nodes:
  - id: team-investigation
    type: Fleet
    config:
      fleet: incident-team
      input: "Investigate outage"
\`\`\`

### Direct Execution
\`\`\`bash
aofctl run fleet incident-team "investigate API latency spike"
\`\`\`

## Best Practices

1. **Compose, don't duplicate** - Reference agents via `ref:`
2. **Define coordination** - Specify agent roles and workflow
3. **Keep focused** - One fleet, one clear purpose
4. **Document workflows** - Explain agent collaboration patterns
