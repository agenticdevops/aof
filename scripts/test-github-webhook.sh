#!/bin/bash
# Test GitHub webhook locally

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing GitHub Webhook Integration${NC}"
echo ""

# Check if server is running
echo -e "${YELLOW}1. Checking if AOF server is running...${NC}"
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Server is running${NC}"
else
    echo -e "${RED}✗ Server is not running${NC}"
    echo "Start the server with: ./target/release/aofctl serve --config config/aof/daemon.yaml"
    exit 1
fi

# Check environment variables
echo -e "\n${YELLOW}2. Checking environment variables...${NC}"
if [ -z "$GITHUB_WEBHOOK_SECRET" ]; then
    echo -e "${RED}✗ GITHUB_WEBHOOK_SECRET not set${NC}"
    echo "Set it with: export GITHUB_WEBHOOK_SECRET=\$(openssl rand -hex 32)"
    exit 1
else
    echo -e "${GREEN}✓ GITHUB_WEBHOOK_SECRET is set${NC}"
fi

if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${YELLOW}⚠ GITHUB_TOKEN not set (API features will be disabled)${NC}"
else
    echo -e "${GREEN}✓ GITHUB_TOKEN is set${NC}"
fi

# Test ping event
echo -e "\n${YELLOW}3. Testing GitHub ping event...${NC}"

# Create HMAC signature
PAYLOAD='{"zen":"AOF test webhook","hook_id":12345678,"hook":{"type":"Repository","id":12345678,"active":true},"repository":{"id":123456,"name":"test-repo","full_name":"test/test-repo","private":false},"sender":{"id":1,"login":"testuser","type":"User"}}'

# Calculate signature (GitHub uses HMAC-SHA256)
SIGNATURE="sha256=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$GITHUB_WEBHOOK_SECRET" | sed 's/^.* //')"

echo "Payload: $PAYLOAD"
echo "Signature: ${SIGNATURE:0:20}..."

# Send webhook
HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/webhook_response.txt -X POST http://localhost:8080/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: ping" \
  -H "X-Hub-Signature-256: $SIGNATURE" \
  -H "X-GitHub-Delivery: $(uuidgen)" \
  -d "$PAYLOAD")

BODY=$(cat /tmp/webhook_response.txt)

if [ "$HTTP_CODE" = "200" ]; then
    echo -e "${GREEN}✓ Ping event successful (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY"
else
    echo -e "${RED}✗ Ping event failed (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY"
    exit 1
fi

# Test pull request event
echo -e "\n${YELLOW}4. Testing GitHub pull_request event...${NC}"

PR_PAYLOAD=$(cat <<'EOF'
{
  "action": "opened",
  "number": 1,
  "pull_request": {
    "id": 1,
    "number": 1,
    "title": "Test PR",
    "body": "This is a test pull request",
    "state": "open",
    "draft": false,
    "merged": false,
    "html_url": "https://github.com/test/repo/pull/1",
    "user": {
      "id": 12345,
      "login": "testuser",
      "type": "User"
    },
    "base": {
      "ref": "main",
      "sha": "abc123"
    },
    "head": {
      "ref": "feature-branch",
      "sha": "def456"
    },
    "additions": 10,
    "deletions": 5,
    "changed_files": 2
  },
  "repository": {
    "id": 123456,
    "name": "repo",
    "full_name": "test/repo",
    "private": false
  },
  "sender": {
    "id": 12345,
    "login": "testuser",
    "type": "User"
  }
}
EOF
)

PR_SIGNATURE="sha256=$(echo -n "$PR_PAYLOAD" | openssl dgst -sha256 -hmac "$GITHUB_WEBHOOK_SECRET" | sed 's/^.* //')"

HTTP_CODE=$(curl -s -w "%{http_code}" -o /tmp/webhook_response.txt -X POST http://localhost:8080/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-GitHub-Event: pull_request" \
  -H "X-Hub-Signature-256: $PR_SIGNATURE" \
  -H "X-GitHub-Delivery: $(uuidgen)" \
  -d "$PR_PAYLOAD")

BODY=$(cat /tmp/webhook_response.txt)

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "202" ]; then
    echo -e "${GREEN}✓ Pull request event successful (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY"
else
    echo -e "${RED}✗ Pull request event failed (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY"
    exit 1
fi

echo -e "\n${GREEN}All tests passed! ✓${NC}"
echo ""
echo "Next steps:"
echo "1. Configure GitHub webhook: https://github.com/<owner>/<repo>/settings/hooks"
echo "2. Use ngrok for local testing: ngrok http 8080"
echo "3. Set Payload URL to: https://your-ngrok-url.ngrok.io/webhook/github"
echo "4. Use the same GITHUB_WEBHOOK_SECRET in GitHub webhook configuration"
