#!/bin/bash
# PR Review Script - Fetches PR, generates AI review, posts to GitHub
# Usage: ./scripts/pr-review.sh <owner/repo> <pr_number>

set -e

REPO="${1:-$GITHUB_REPO}"
PR_NUM="${2:-$GITHUB_PR_NUMBER}"

if [ -z "$REPO" ] || [ -z "$PR_NUM" ]; then
  echo "Usage: $0 <owner/repo> <pr_number>"
  echo "Or set GITHUB_REPO and GITHUB_PR_NUMBER environment variables"
  exit 1
fi

echo "=== AOF PR Review ==="
echo "Repository: $REPO"
echo "PR: #$PR_NUM"

# Authenticate with GitHub
echo "$GITHUB_TOKEN" | gh auth login --with-token 2>/dev/null || true

# Get PR info
echo "Fetching PR info..."
TITLE=$(gh pr view "$PR_NUM" --repo "$REPO" --json title -q '.title')
AUTHOR=$(gh pr view "$PR_NUM" --repo "$REPO" --json author -q '.author.login')
ADDITIONS=$(gh pr view "$PR_NUM" --repo "$REPO" --json additions -q '.additions')
DELETIONS=$(gh pr view "$PR_NUM" --repo "$REPO" --json deletions -q '.deletions')
CHANGED=$(gh pr view "$PR_NUM" --repo "$REPO" --json changedFiles -q '.changedFiles')
BODY=$(gh pr view "$PR_NUM" --repo "$REPO" --json body -q '.body // "No description"')

echo "Title: $TITLE"
echo "Author: @$AUTHOR"
echo "Changes: +$ADDITIONS/-$DELETIONS in $CHANGED files"

# Get the diff
echo "Fetching diff..."
DIFF=$(gh pr diff "$PR_NUM" --repo "$REPO" 2>/dev/null || echo "Could not fetch diff")

# Build prompt
cat > /tmp/pr-review-prompt.txt << EOF
You are an expert code reviewer. Review this pull request thoroughly.

## PR Information
- Repository: $REPO
- PR #$PR_NUM: $TITLE
- Author: @$AUTHOR
- Changes: +$ADDITIONS/-$DELETIONS in $CHANGED files

## Description
$BODY

## Code Changes
\`\`\`diff
$DIFF
\`\`\`

Provide a comprehensive code review (under 500 words) with:
1. **Overall Assessment**: Approve ✅ / Request Changes ⚠️ / Comment 💬
2. **Summary**: Brief overview of the changes
3. **Analysis**: What the changes do and their impact
4. **Concerns**: Any issues, security risks, or improvements needed
5. **Verdict**: Final recommendation

Use markdown formatting. Be constructive and specific.
EOF

echo "Generating review with Gemini..."

# Call Gemini API using Python (handles JSON properly)
python3 << 'PYEOF'
import json
import urllib.request
import os
import sys

api_key = os.environ.get('GOOGLE_API_KEY')
if not api_key:
    print("Error: GOOGLE_API_KEY not set")
    sys.exit(1)

with open('/tmp/pr-review-prompt.txt', 'r') as f:
    prompt = f.read()

data = {"contents": [{"parts": [{"text": prompt}]}]}
url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={api_key}"

req = urllib.request.Request(url,
    data=json.dumps(data).encode('utf-8'),
    headers={'Content-Type': 'application/json'})

try:
    with urllib.request.urlopen(req, timeout=60) as response:
        result = json.loads(response.read().decode('utf-8'))
        review = result['candidates'][0]['content']['parts'][0]['text']

        review_with_sig = review + "\n\n---\n_🤖 Automated review by [AOF Bot](https://docs.aof.sh) | Powered by Google Gemini 2.5 Flash_"

        with open('/tmp/pr-review.md', 'w') as f:
            f.write(review_with_sig)

        print("Review generated!")
except Exception as e:
    print(f"API Error: {e}")
    sys.exit(1)
PYEOF

# Post to GitHub
if [ -f /tmp/pr-review.md ]; then
  echo "Posting review to GitHub..."
  gh pr comment "$PR_NUM" --repo "$REPO" --body-file /tmp/pr-review.md
  echo ""
  echo "✅ Review posted successfully!"
  echo "View at: https://github.com/$REPO/pull/$PR_NUM"
else
  echo "❌ Failed to generate review"
  exit 1
fi
