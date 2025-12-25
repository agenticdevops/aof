#!/bin/bash

# Test output schema rendering manually
TEST_JSON='```json
{
  "containers": [
    {
      "image": "alpine/socat",
      "name": "keen_yonath",
      "status": "Running",
      "uptime": "14 hours"
    },
    {
      "image": "postgres:16-alpine",
      "name": "test-db",
      "status": "Running",
      "uptime": "2 days"
    }
  ]
}
```'

echo "$TEST_JSON" > /tmp/test-llm-output.txt

# Run with debug output schema container-list
./target/release/aofctl run agent examples/quickstart/docker-health-agent.yaml \
  --input "ignore this" \
  --output-schema container-list 2>&1 | tail -50
