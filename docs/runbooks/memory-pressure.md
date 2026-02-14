# Runbook: Memory Pressure

## Symptoms
- System health in Degraded or Critical state
- Memory usage > 70% (warning) or > 90% (critical)
- Performance degradation

## Impact
- Slower agent execution
- Potential OOM kills
- Service instability

## Severity
**P1** if Critical, **P2** if Degraded

## Investigation Steps

1. **Check system health:**
   ```bash
   curl localhost:8080/health
   ```

2. **Check memory usage:**
   ```bash
   ps aux | grep aofctl
   top -p $(pgrep aofctl)
   ```

3. **Check agent count:**
   ```bash
   curl localhost:8080/metrics | grep agents_active
   ```

4. **Check event queue depth:**
   ```bash
   curl localhost:8080/metrics | grep events_emitted
   ```

## Mitigation (Immediate)

1. **Reduce agent count:**
   - Lower bulkhead max_concurrent_agents
   - Restart with lower limits

2. **Clear caches:**
   - Degradation engine auto-clears on detection
   - Manual restart if needed

3. **Increase resource limits:**
   ```bash
   # Docker
   docker update --memory 4G aof-daemon

   # Systemd
   # Update MemoryMax in service file
   ```

## Resolution (Permanent)

1. Investigate memory leaks (growing without bound)
2. Optimize agent memory usage
3. Add memory limits per agent
4. Scale horizontally if needed

## Escalation
Escalate if memory continues growing after mitigation.

## Prevention
- Monitor memory trends
- Set up 70% threshold alerts
- Regular memory profiling
