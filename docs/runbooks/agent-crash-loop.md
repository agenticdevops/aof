# Runbook: Agent Crash Loop

## Symptoms
- Agent restarts repeatedly
- `aof_agent_restarts_total` metric increasing rapidly
- Logs show repeated panic/error messages

## Impact
- Affected agent unavailable
- Resource consumption from restart attempts
- Circuit breaker may open

## Severity
**P2** - Service degraded but not down

## Investigation Steps

1. **Check logs for crash cause:**
   ```bash
   aofctl logs | grep -A 10 "agent_id=<agent-id>"
   ```

2. **Check restart count:**
   ```bash
   curl localhost:8080/metrics | grep agent_restarts_total
   ```

3. **Check circuit breaker state:**
   ```bash
   # Circuit should open after 5 failures
   aofctl status
   ```

## Mitigation (Immediate)

1. **Disable problematic agent:**
   ```bash
   # Remove from AGENTS.md or mark disabled
   # Restart daemon
   ```

2. **Reset circuit breaker if needed:**
   ```bash
   # Automatic after timeout, or restart daemon
   pkill aofctl && aofctl serve &
   ```

## Resolution (Permanent)

1. Fix root cause (missing dependency, bad prompt, resource limit)
2. Add error handling to agent task
3. Update agent configuration
4. Re-enable agent and monitor

## Escalation
Escalate to engineering if crash cause unknown after 30 minutes.

## Prevention
- Add validation to agent prompts
- Set resource limits
- Add pre-flight checks
