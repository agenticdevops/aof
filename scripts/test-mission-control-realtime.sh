#!/usr/bin/env bash
#
# Integration test for Mission Control real-time event pipeline.
#
# Tests the end-to-end flow:
#   API POST -> EventBroadcaster -> WebSocket -> Client
#
# Requirements:
#   - aofctl binary built (cargo build --release or debug)
#   - websocat (optional, falls back to curl-only mode)
#   - curl
#   - jq
#
# Usage:
#   ./scripts/test-mission-control-realtime.sh
#
# Exit codes:
#   0 = all tests passed
#   1 = one or more tests failed

set -euo pipefail

# Configuration
PORT="${AOF_TEST_PORT:-18080}"
HOST="127.0.0.1"
BASE_URL="http://${HOST}:${PORT}"
WS_URL="ws://${HOST}:${PORT}/ws"
AOFCTL="${AOFCTL_BIN:-./target/debug/aofctl}"
TIMEOUT_SECS=10
WS_LOG="/tmp/aof-ws-events-$$.log"
SERVER_LOG="/tmp/aof-server-$$.log"
WS_FIFO="/tmp/aof-ws-fifo-$$"
SERVER_PID=""
WS_PID=""
FIFO_PID=""

# Colors (only if terminal supports them)
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  RED='\033[0;31m'
  YELLOW='\033[0;33m'
  BLUE='\033[0;34m'
  NC='\033[0m'
else
  GREEN='' RED='' YELLOW='' BLUE='' NC=''
fi

# Counters
PASS=0
FAIL=0
TOTAL=0

# ============================================================================
# Helpers
# ============================================================================

log_info() {
  echo -e "${BLUE}[INFO]${NC} $*"
}

log_pass() {
  echo -e "${GREEN}[PASS]${NC} $*"
  PASS=$((PASS + 1))
  TOTAL=$((TOTAL + 1))
}

log_fail() {
  echo -e "${RED}[FAIL]${NC} $*"
  FAIL=$((FAIL + 1))
  TOTAL=$((TOTAL + 1))
}

log_skip() {
  echo -e "${YELLOW}[SKIP]${NC} $*"
  TOTAL=$((TOTAL + 1))
}

stop_ws() {
  if [ -n "$FIFO_PID" ] && kill -0 "$FIFO_PID" 2>/dev/null; then
    kill "$FIFO_PID" 2>/dev/null || true
    wait "$FIFO_PID" 2>/dev/null || true
    FIFO_PID=""
  fi
  if [ -n "$WS_PID" ] && kill -0 "$WS_PID" 2>/dev/null; then
    kill "$WS_PID" 2>/dev/null || true
    wait "$WS_PID" 2>/dev/null || true
    WS_PID=""
  fi
}

start_ws() {
  stop_ws
  > "$WS_LOG"
  mkfifo "$WS_FIFO" 2>/dev/null || true
  sleep 999 > "$WS_FIFO" &
  FIFO_PID=$!
  websocat -t "$WS_URL" < "$WS_FIFO" > "$WS_LOG" 2>/dev/null &
  WS_PID=$!
  sleep 2
}

cleanup() {
  log_info "Cleaning up..."
  stop_ws
  if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  rm -f "$WS_LOG" "$SERVER_LOG" "$WS_FIFO"
}

trap cleanup EXIT

wait_for_server() {
  local max_attempts=30
  local attempt=0
  while [ $attempt -lt $max_attempts ]; do
    if curl -sf "${BASE_URL}/health" > /dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
    attempt=$((attempt + 1))
  done
  return 1
}

has_websocat() {
  command -v websocat > /dev/null 2>&1
}

# ============================================================================
# Phase 1: Start Server
# ============================================================================

log_info "========================================="
log_info "Mission Control Real-Time Integration Test"
log_info "========================================="
echo ""

# Check if aofctl binary exists
if [ ! -f "$AOFCTL" ]; then
  log_info "aofctl binary not found at $AOFCTL, building..."
  cargo build -p aofctl 2>/dev/null
  if [ ! -f "$AOFCTL" ]; then
    log_fail "Failed to build aofctl"
    exit 1
  fi
fi

log_info "Starting aofctl serve on port ${PORT}..."
$AOFCTL serve --port "$PORT" > "$SERVER_LOG" 2>&1 &
SERVER_PID=$!

log_info "Waiting for server to become ready..."
if ! wait_for_server; then
  log_fail "Server failed to start within ${TIMEOUT_SECS}s"
  cat "$SERVER_LOG"
  exit 1
fi
log_pass "Server started (PID: ${SERVER_PID})"

# ============================================================================
# Phase 2: Health Check
# ============================================================================

echo ""
log_info "--- Health Check ---"

HEALTH=$(curl -sf "${BASE_URL}/health")
if echo "$HEALTH" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
  log_pass "Health endpoint returns healthy"
else
  log_fail "Health endpoint unhealthy: $HEALTH"
fi

# ============================================================================
# Phase 3: Test Event Endpoint
# ============================================================================

echo ""
log_info "--- Test Event Endpoint ---"

# Test: emit agent_started event
RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"k8s-monitor","event_type":"agent_started","details":{}}')

if echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
  EVENT_ID=$(echo "$RESPONSE" | jq -r '.event_id')
  log_pass "Emit agent_started event (event_id: ${EVENT_ID:0:8}...)"
else
  log_fail "Failed to emit agent_started event: $RESPONSE"
fi

# Test: emit agent_completed event
RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"k8s-monitor","event_type":"agent_completed","details":{}}')

if echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
  log_pass "Emit agent_completed event"
else
  log_fail "Failed to emit agent_completed event: $RESPONSE"
fi

# Test: emit agent_error event
RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"k8s-monitor","event_type":"agent_error","details":{"reason":"test error"}}')

if echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
  log_pass "Emit agent_error event"
else
  log_fail "Failed to emit agent_error event: $RESPONSE"
fi

# Test: emit thinking event
RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"log-analyzer","event_type":"thinking","details":{}}')

if echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
  log_pass "Emit thinking event"
else
  log_fail "Failed to emit thinking event: $RESPONSE"
fi

# Test: emit tool_executing event
RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"deployer","event_type":"tool_executing","details":{"tool":"kubectl"}}')

if echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
  log_pass "Emit tool_executing event"
else
  log_fail "Failed to emit tool_executing event: $RESPONSE"
fi

# Test: emit tool_completed event
RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"deployer","event_type":"tool_completed","details":{"tool":"kubectl"}}')

if echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
  log_pass "Emit tool_completed event"
else
  log_fail "Failed to emit tool_completed event: $RESPONSE"
fi

# Test: validation - empty agent_id
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "${BASE_URL}/api/test/emit-event" \
  -H 'Content-Type: application/json' \
  -d '{"agent_id":"","event_type":"agent_started","details":{}}')

if [ "$RESPONSE" = "400" ]; then
  log_pass "Rejects empty agent_id (400)"
else
  log_fail "Should reject empty agent_id, got HTTP $RESPONSE"
fi

# ============================================================================
# Phase 4: WebSocket Event Delivery (if websocat available)
# ============================================================================

echo ""
log_info "--- WebSocket Event Delivery ---"

if has_websocat; then
  # Start WebSocket listener in background, capturing events
  start_ws

  # Emit a unique event
  UNIQUE_AGENT="ws-test-$(date +%s)"
  curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
    -H 'Content-Type: application/json' \
    -d "{\"agent_id\":\"${UNIQUE_AGENT}\",\"event_type\":\"agent_started\",\"details\":{}}" > /dev/null

  # Wait for event to arrive on WebSocket
  sleep 2

  # Check if event arrived on WebSocket
  if grep -q "$UNIQUE_AGENT" "$WS_LOG" 2>/dev/null; then
    log_pass "Event delivered via WebSocket (agent: ${UNIQUE_AGENT})"
  else
    log_fail "Event not received on WebSocket within 1s"
  fi

  # Verify event is valid JSON with expected fields
  if grep "$UNIQUE_AGENT" "$WS_LOG" 2>/dev/null | head -1 | jq -e '.event_id and .agent_id and .activity' > /dev/null 2>&1; then
    log_pass "WebSocket event is valid CoordinationEvent JSON"
  else
    log_fail "WebSocket event has invalid format"
  fi

  # Verify activity_type field exists (not type)
  if grep "$UNIQUE_AGENT" "$WS_LOG" 2>/dev/null | head -1 | jq -e '.activity.activity_type == "Started"' > /dev/null 2>&1; then
    log_pass "WebSocket event has correct activity_type: Started"
  else
    log_fail "WebSocket event missing or wrong activity_type"
  fi

  # Clean up WebSocket connection
  stop_ws
else
  log_skip "WebSocket delivery test (websocat not installed: brew install websocat)"
  log_skip "WebSocket event format test (requires websocat)"
  log_skip "WebSocket activity_type test (requires websocat)"
fi

# ============================================================================
# Phase 5: Tasks API Events
# ============================================================================

echo ""
log_info "--- Tasks API Events ---"

# Start fresh WebSocket listener if websocat available
if has_websocat; then
  start_ws
fi

# Create a task
CREATE_RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/tasks" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Integration test task","description":"Created by realtime test"}')

if echo "$CREATE_RESPONSE" | jq -e '.id and .title == "Integration test task"' > /dev/null 2>&1; then
  TASK_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
  log_pass "Created task (id: ${TASK_ID:0:8}...)"
else
  log_fail "Failed to create task: $CREATE_RESPONSE"
  TASK_ID=""
fi

# Move the task
if [ -n "$TASK_ID" ]; then
  MOVE_RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/tasks/move" \
    -H 'Content-Type: application/json' \
    -d "{\"taskId\":\"${TASK_ID}\",\"newLane\":\"in-progress\",\"version\":1}")

  if echo "$MOVE_RESPONSE" | jq -e '.success == true' > /dev/null 2>&1; then
    log_pass "Moved task to in-progress"
  else
    log_fail "Failed to move task: $MOVE_RESPONSE"
  fi
fi

# Verify task events on WebSocket
if has_websocat; then
  sleep 2
  if grep -q "TASK_CREATED" "$WS_LOG" 2>/dev/null; then
    log_pass "TASK_CREATED event received on WebSocket"
  else
    log_fail "TASK_CREATED event not received on WebSocket"
  fi

  if grep -q "TASK_MOVED" "$WS_LOG" 2>/dev/null; then
    log_pass "TASK_MOVED event received on WebSocket"
  else
    log_fail "TASK_MOVED event not received on WebSocket"
  fi

  stop_ws
else
  log_skip "TASK_CREATED WebSocket event (requires websocat)"
  log_skip "TASK_MOVED WebSocket event (requires websocat)"
fi

# ============================================================================
# Phase 6: Chat API Events
# ============================================================================

echo ""
log_info "--- Chat API Events ---"

# Start fresh WebSocket listener if websocat available
if has_websocat; then
  start_ws
fi

# Send a chat message
CHAT_RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/chat/messages" \
  -H 'Content-Type: application/json' \
  -d '{"content":"Hello from integration test!","senderId":"test-user","senderName":"Test User"}')

if echo "$CHAT_RESPONSE" | jq -e '.id and .content == "Hello from integration test!"' > /dev/null 2>&1; then
  log_pass "Sent chat message"
else
  log_fail "Failed to send chat message: $CHAT_RESPONSE"
fi

# Verify chat event on WebSocket
if has_websocat; then
  sleep 2
  if grep -q "chat_message" "$WS_LOG" 2>/dev/null; then
    log_pass "CHAT_MESSAGE event received on WebSocket"
  else
    log_fail "CHAT_MESSAGE event not received on WebSocket"
  fi

  stop_ws
else
  log_skip "CHAT_MESSAGE WebSocket event (requires websocat)"
fi

# Verify chat message appears in history
HISTORY=$(curl -sf "${BASE_URL}/api/chat/messages")
if echo "$HISTORY" | jq -e '.[] | select(.content == "Hello from integration test!")' > /dev/null 2>&1; then
  log_pass "Chat message persisted in history"
else
  log_fail "Chat message not found in history"
fi

# ============================================================================
# Phase 7: Multiple Event Types
# ============================================================================

echo ""
log_info "--- Multiple Event Types ---"

# Emit all supported event types and verify each returns 200
EVENT_TYPES=("agent_started" "agent_completed" "agent_error" "task_assigned" "tool_called" "tool_completed" "tool_failed" "thinking" "info" "warning")
ALL_EMIT_OK=true

for EVENT_TYPE in "${EVENT_TYPES[@]}"; do
  RESPONSE=$(curl -sf -X POST "${BASE_URL}/api/test/emit-event" \
    -H 'Content-Type: application/json' \
    -d "{\"agent_id\":\"multi-test\",\"event_type\":\"${EVENT_TYPE}\",\"details\":{}}")

  if ! echo "$RESPONSE" | jq -e '.emitted == true' > /dev/null 2>&1; then
    ALL_EMIT_OK=false
    break
  fi
done

if [ "$ALL_EMIT_OK" = true ]; then
  log_pass "All ${#EVENT_TYPES[@]} event types emit successfully"
else
  log_fail "Some event types failed to emit"
fi

# ============================================================================
# Results
# ============================================================================

echo ""
log_info "========================================="
log_info "Results: ${PASS} passed, ${FAIL} failed out of ${TOTAL} tests"
log_info "========================================="

if [ "$FAIL" -gt 0 ]; then
  echo ""
  log_info "Server log (last 20 lines):"
  tail -20 "$SERVER_LOG" 2>/dev/null || true
  exit 1
else
  exit 0
fi
