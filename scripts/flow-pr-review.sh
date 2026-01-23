#!/bin/bash
# Flow PR Review Script - Called by AgentFlow
# Parses trigger data and calls the actual review script

set -e

echo "=== AOF PR Review Flow ==="
echo ""

# Parse trigger data JSON to extract repo and PR number
# The AOF_TRIGGER_DATA contains the full trigger data JSON like:
# {"event":{"channel_id":"owner/repo#42","metadata":{"repo_full_name":"owner/repo","pr_number":42},...},...}

if [ -z "$AOF_TRIGGER_DATA" ]; then
  echo "Error: AOF_TRIGGER_DATA not set"
  exit 1
fi

# Use Python to parse the JSON (more reliable than jq)
eval "$(python3 << 'PYEOF'
import json
import os
import sys
import re

try:
    data = json.loads(os.environ.get('AOF_TRIGGER_DATA', '{}'))
    event = data.get('event', {})
    meta = event.get('metadata', {})

    # Method 1: Get from metadata (most reliable for GitHub events)
    repo = meta.get('repo_full_name', '')
    pr_num = meta.get('pr_number', '')

    # Method 2: Parse from channel_id (format: "owner/repo#number")
    if not repo or not pr_num:
        channel_id = event.get('channel_id', '')
        if '#' in str(channel_id):
            parts = str(channel_id).split('#')
            if not repo:
                repo = parts[0]
            if not pr_num and len(parts) > 1:
                pr_num = parts[1]

    # Method 3: Parse from text (format: "pr:opened:base:head #42 Title - owner/repo")
    if not repo or not pr_num:
        text = event.get('text', '')
        if not pr_num:
            pr_match = re.search(r'#(\d+)', text)
            if pr_match:
                pr_num = pr_match.group(1)
        if not repo:
            repo_match = re.search(r' - ([a-zA-Z0-9_-]+/[a-zA-Z0-9_.-]+)\s*$', text)
            if repo_match:
                repo = repo_match.group(1)

    # Convert pr_num to string if it's an int
    if isinstance(pr_num, int):
        pr_num = str(pr_num)

    print(f"export REPO='{repo}'")
    print(f"export PR_NUM='{pr_num}'")

except Exception as e:
    print(f"echo 'Parse error: {e}'", file=sys.stderr)
    print("export REPO=''")
    print("export PR_NUM=''")
PYEOF
)"

if [ -z "$REPO" ] || [ -z "$PR_NUM" ]; then
  echo "Error: Could not determine repository and PR number"
  echo ""
  echo "Trigger data preview:"
  echo "${AOF_TRIGGER_DATA:0:500}..."
  exit 1
fi

echo "Repository: $REPO"
echo "PR Number: $PR_NUM"
echo ""

# Run the actual review script
exec ./scripts/pr-review.sh "$REPO" "$PR_NUM"
