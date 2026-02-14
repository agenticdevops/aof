#!/bin/bash
# Quick 5-minute test of AOF with Google Gemini

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "AOF Quick Test (5 minutes)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check API key
if [ -z "$GOOGLE_API_KEY" ]; then
    echo "❌ Error: GOOGLE_API_KEY not set"
    echo "   Run: export GOOGLE_API_KEY='your-key-here'"
    exit 1
fi

echo "✓ GOOGLE_API_KEY is set"

# Navigate to repo
cd "$(dirname "$0")/.."
echo "✓ Working directory: $(pwd)"

# Run quick test
echo ""
echo "Running agent: quick-test"
echo "Prompt: Tell me a short joke about programming"
echo ""

cargo run -p aofctl -- run agent gojo/agents/quick-test.yaml \
    --prompt "Tell me a short joke about programming"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Test completed successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
