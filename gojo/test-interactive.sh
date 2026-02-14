#!/bin/bash
# Interactive multi-turn conversation test

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "AOF Interactive Test (Multi-turn Conversation)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check API key
if [ -z "$GOOGLE_API_KEY" ]; then
    echo "❌ Error: GOOGLE_API_KEY not set"
    exit 1
fi

cd "$(dirname "$0")/.."

echo "Starting interactive agent: general-assistant"
echo "Type your questions below. Press Ctrl+C to exit."
echo ""

cargo run -p aofctl -- run agent gojo/agents/quick-test.yaml --interactive
