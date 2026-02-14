# Runbook: High Error Rate

## Symptoms
- `aof_agent_executions_total{status='error'}` above threshold
- Error rate > 0.1% (SLO breach)
- Users reporting task failures

## Impact
- Degraded service quality
- SLO violation
- Error budget consumption

## Severity
**P1** - SLO breach

## Investigation Steps

1. **Check which agents are failing:**
   ```bash
   curl localhost:8080/metrics | grep 'agent_executions_total{.*error'
   ```

2. **Check LLM provider status:**
   ```bash
   # Check Anthropic status page
   # Check API key validity
   ```

3. **Check MCP server connectivity:**
   ```bash
   aofctl mcp list
   ```

4. **Check circuit breaker status:**
   ```bash
   curl localhost:8080/metrics | grep circuit_breaker
   ```

## Mitigation (Immediate)

1. **Switch to degraded mode:**
   - Reduce agent concurrency
   - Skip non-essential tasks
   - Enable circuit breakers

2. **Fallback to alternate provider if LLM issue:**
   ```bash
   export AOF_LLM_PROVIDER=openai
   ```

## Resolution (Permanent)

1. Fix upstream service if external
2. Update agent prompts if validation failing
3. Scale resources if capacity issue
4. Update error handling

## Escalation
Escalate immediately if error rate > 1% or lasting > 15 minutes.

## Prevention
- Monitor error rate continuously
- Set up alerts at 0.05% threshold
- Maintain circuit breaker configs
