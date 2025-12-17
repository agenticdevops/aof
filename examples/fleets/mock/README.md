# Mock Fleet Examples

This directory contains **mock/demo** fleet examples that simulate real infrastructure for testing and learning purposes.

## Purpose

Mock examples allow you to:
- **Test AOF features** without infrastructure dependencies
- **Learn fleet patterns** before deploying to production
- **Demo capabilities** to stakeholders
- **Develop and iterate** quickly without cost

## Available Mock Fleets

| Fleet | Description | Prerequisites |
|-------|-------------|---------------|
| `multi-model-rca-mock.yaml` | Multi-model RCA with simulated observability data | Google API key only |

## How Mock Examples Work

Mock examples use LLMs to **simulate** what real data would look like:

```
┌─────────────────────────────────────────────────────────────────┐
│                      MOCK FLEET                                 │
│                                                                 │
│  Instead of:                    Mock agents do:                 │
│  ─────────────                  ────────────────                │
│  Query Prometheus         →     Simulate realistic metrics      │
│  Query Loki logs          →     Generate plausible log entries  │
│  Run kubectl              →     Describe expected K8s state     │
│                                                                 │
│  Result: Realistic demo output without real infrastructure      │
└─────────────────────────────────────────────────────────────────┘
```

## Usage

```bash
# Run a mock fleet
aofctl run fleet examples/fleets/mock/multi-model-rca-mock.yaml \
  --input "Investigate: API returning 500 errors since deployment"

# Try different scenarios
aofctl run fleet examples/fleets/mock/multi-model-rca-mock.yaml \
  --input "Investigate: High memory usage causing pod crashes"

aofctl run fleet examples/fleets/mock/multi-model-rca-mock.yaml \
  --input "Investigate: Database connection timeouts"
```

## Cost

Mock examples are designed to be **cheap to run**:

| Tier | Agents | Model | Est. Cost |
|------|--------|-------|-----------|
| 1 (Simulators) | 3 | Gemini Flash | ~$0.01 |
| 2 (Reasoning) | 2 | Gemini Pro/Flash | ~$0.05 |
| 3 (Coordinator) | 1 | Gemini Pro | ~$0.02 |
| **Total** | 6 | - | **~$0.08** |

## When to Use Mock vs Real

| Scenario | Use Mock | Use Real |
|----------|----------|----------|
| Learning AOF | ✅ | |
| Demos to stakeholders | ✅ | |
| Testing fleet configurations | ✅ | |
| CI/CD pipeline tests | ✅ | |
| Actual incident response | | ✅ |
| Production RCA | | ✅ |
| Training with real data | | ✅ |

## Creating Your Own Mock Fleets

To create a mock version of any fleet:

1. Replace data-fetching agents with "simulator" agents
2. Update instructions to generate realistic simulated data
3. Keep the same tier structure and consensus configuration
4. Add `mock: "true"` label for clarity

Example simulator agent:

```yaml
- name: metrics-simulator
  tier: 1
  spec:
    model: google:gemini-2.0-flash
    instructions: |
      You are a Metrics Simulator.
      Given an incident description, SIMULATE what metrics would show.
      Generate realistic values that correlate with the incident type.
      Output structured JSON matching the real collector format.
```

## See Also

- **Real examples**: `../real/` - Use actual infrastructure
- **Multi-Model RCA Tutorial**: `../../docs/tutorials/multi-model-rca.md`
- **Fleet Concepts**: `../../docs/concepts/fleets.md`
