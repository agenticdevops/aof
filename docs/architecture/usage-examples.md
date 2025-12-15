# AOF Memory and RAG System - Usage Examples

## Example 1: Using the K8s Expert Agent

### CLI Interaction

```bash
# Deploy the agent
kubectl apply -f k8s-expert-agent.yaml

# Query the agent
aof agent query k8s-expert \
  --message "Pod crash looping in production namespace. How do I debug this?" \
  --conversation-id "incident-2024-001"

# Response will include:
# - Relevant runbooks from memory
# - Similar past incidents
# - Current pod status from K8s API
# - Step-by-step troubleshooting guide
```

### Example Query 1: Pod CrashLoopBackOff

**User Query:**
```
I have a pod in CrashLoopBackOff in the production namespace.
The pod name is web-app-7d9f6b8c4-xyz. How do I troubleshoot this?
```

**Agent Response:**
```
## Problem Summary
Pod `web-app-7d9f6b8c4-xyz` in namespace `production` is in CrashLoopBackOff state,
indicating the container is repeatedly crashing and Kubernetes is backing off on restart attempts.

## Immediate Actions
1. Check pod logs for crash reason:
   kubectl logs web-app-7d9f6b8c4-xyz -n production --previous

2. Check recent events:
   kubectl describe pod web-app-7d9f6b8c4-xyz -n production

## Root Cause Analysis
Based on similar incidents in our knowledge base (Incident #INC-2024-089), common causes include:
- Application configuration errors (45% of cases)
- Missing dependencies or environment variables (30%)
- Resource constraints (memory/CPU limits) (15%)
- External service unavailability (10%)

## Resolution Steps

### Step 1: Inspect Container Logs
\`\`\`bash
# Get logs from the crashed container
kubectl logs web-app-7d9f6b8c4-xyz -n production --previous

# If multiple containers, specify container name
kubectl logs web-app-7d9f6b8c4-xyz -c web-app -n production --previous
\`\`\`

### Step 2: Check Pod Description
\`\`\`bash
kubectl describe pod web-app-7d9f6b8c4-xyz -n production
\`\`\`
Look for:
- Events section for error messages
- Container state and exit code
- Resource limits vs requests

### Step 3: Verify Configuration
\`\`\`bash
# Check ConfigMaps
kubectl get configmap -n production -l app=web-app

# Check Secrets
kubectl get secret -n production -l app=web-app

# Compare with working deployment in staging
kubectl diff -f deployment.yaml
\`\`\`

### Step 4: Check Resource Metrics
\`\`\`bash
kubectl top pod web-app-7d9f6b8c4-xyz -n production
kubectl top node
\`\`\`

### Step 5: Analyze Exit Code
Common exit codes:
- Exit 0: Clean shutdown (check if app is exiting on purpose)
- Exit 1: Generic error (check application logs)
- Exit 137: OOMKilled (increase memory limits)
- Exit 139: Segmentation fault (application bug)

## Current System State
📊 **Active Incidents:** 2 incidents currently in PagerDuty
- INC-456: Database connection pool exhaustion
- INC-457: Ingress controller high latency

⚠️ **Recent Deployments:**
- web-app v2.4.3 deployed 15 minutes ago to production

🔥 **Firing Alerts:**
- PodCrashLooping: web-app in production

## Prevention
1. **Pre-deployment Checks:**
   - Test configuration in staging first
   - Verify all dependencies are available
   - Run integration tests

2. **Monitoring:**
   - Set up alerts for CrashLoopBackOff
   - Monitor application startup time
   - Track error rates in logs

3. **Resource Management:**
   - Set appropriate memory/CPU limits
   - Use horizontal pod autoscaling
   - Monitor resource usage trends

## References
- [Runbook: Pod CrashLoopBackOff Troubleshooting](runbooks/k8s/pod-crashloop.md)
- [K8s Docs: Debug Pods](https://kubernetes.io/docs/tasks/debug/debug-application/debug-pods/)
- [Past Incident: INC-2024-089](confluence/postmortems/2024-089.md) - Similar issue with database connection

## Next Steps
If the issue persists after following these steps:
1. Collect all logs and diagnostics
2. Check if rollback is needed: `kubectl rollout undo deployment/web-app -n production`
3. Escalate to L2 on-call: +1-555-0102
```

### Example Query 2: Performance Investigation

**User Query:**
```
Our API response times have increased from 100ms to 2000ms over the past hour.
What could be causing this and how do I investigate?
```

**Agent Response:**
```
## Problem Summary
API latency has degraded 20x (100ms → 2000ms) in the past hour. This requires immediate
investigation as it's affecting user experience.

## Immediate Actions
1. Check if there's an ongoing deployment:
   kubectl rollout status deployment/api-server -n production

2. Query Prometheus for error rates and resource usage:
   (Commands provided with your configured prometheus-query tool)

## Current System State
📊 **Recent Deployments:**
- api-server v3.2.1 deployed 45 minutes ago ⚠️ TIMING MATCHES ISSUE
- database-proxy v1.5.0 deployed 2 hours ago

🔥 **Firing Alerts:**
- HighRequestLatency: api-server p95 latency > 1s
- HighDatabaseConnections: connection pool 95% utilized

⚠️ **Active Incidents:** None currently in PagerDuty

## Root Cause Analysis
Based on timing correlation and similar incidents:

**High Probability (70%):**
- New deployment v3.2.1 introduced performance regression
- Similar to INC-2024-134: Database query optimization issue

**Medium Probability (20%):**
- Database connection pool exhaustion
- Increased load or traffic spike

**Low Probability (10%):**
- Infrastructure issue (network, node resources)

## Investigation Steps

### Step 1: Verify Recent Deployment
\`\`\`bash
# Check deployment history
kubectl rollout history deployment/api-server -n production

# Check if deployment is still progressing
kubectl rollout status deployment/api-server -n production

# Compare resource usage before/after deployment
kubectl top pods -n production -l app=api-server
\`\`\`

### Step 2: Analyze Application Metrics
Query your monitoring system for:
- Request rate (QPS)
- Error rate (5xx responses)
- P50, P95, P99 latencies
- Database query times
- External API call times

### Step 3: Check Database Performance
\`\`\`bash
# Check database connections
kubectl exec -it api-server-xxx -n production -- \
  psql -c "SELECT count(*) FROM pg_stat_activity;"

# Check for slow queries
kubectl logs api-server-xxx -n production | grep "slow query"
\`\`\`

### Step 4: Inspect Application Logs
\`\`\`bash
# Get recent logs with timing information
kubectl logs -n production -l app=api-server --since=1h | grep -E "ERROR|WARN|timeout"

# Check for connection errors
kubectl logs -n production -l app=api-server --since=1h | grep -E "connection|timeout|refused"
\`\`\`

### Step 5: Compare with Baseline
From our knowledge base, normal metrics for api-server:
- Request rate: 1000-2000 QPS
- P95 latency: 50-100ms
- Error rate: <0.1%
- Database connections: 20-50 active

## Resolution Options

### Option 1: Rollback (Recommended if regression confirmed)
\`\`\`bash
# Rollback to previous version
kubectl rollout undo deployment/api-server -n production

# Monitor rollback
kubectl rollout status deployment/api-server -n production

# Verify metrics improve
\`\`\`

### Option 2: Scale Horizontally (If load-related)
\`\`\`bash
# Temporarily increase replicas
kubectl scale deployment/api-server -n production --replicas=10

# Enable autoscaling
kubectl autoscale deployment/api-server -n production \
  --min=5 --max=20 --cpu-percent=70
\`\`\`

### Option 3: Optimize Database Connections (If connection pool issue)
Check application config for:
- Connection pool size
- Connection timeout settings
- Query optimization opportunities

## Prevention
1. **Pre-deployment Testing:**
   - Load test in staging with production-like traffic
   - Run performance benchmarks
   - Compare metrics with production baseline

2. **Gradual Rollout:**
   - Use canary deployments (10% → 50% → 100%)
   - Monitor metrics at each stage
   - Automated rollback on SLO violations

3. **Monitoring & Alerts:**
   - Alert on latency >500ms
   - Alert on error rate >0.5%
   - Dashboard for request latency trends

## References
- [Runbook: Performance Degradation Response](runbooks/performance/degradation.md)
- [Past Incident: INC-2024-134](confluence/postmortems/2024-134.md) - Database query regression
- [K8s Docs: Horizontal Pod Autoscaling](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)

## Escalation
⚠️ **This is a SEV2 incident affecting production**

If not resolved in 15 minutes:
1. Create PagerDuty incident
2. Notify #incidents Slack channel
3. Escalate to L2: +1-555-0102
4. Consider initiating war room
```

## Example 2: Conversational Memory in Action

```bash
# First query
aof agent query k8s-expert \
  --message "What is a StatefulSet?" \
  --conversation-id "learning-session-1"

# Follow-up question (uses conversational memory)
aof agent query k8s-expert \
  --message "What's the difference between that and a Deployment?" \
  --conversation-id "learning-session-1"
# Agent knows "that" refers to StatefulSet from previous message

# Another follow-up
aof agent query k8s-expert \
  --message "Show me an example YAML for a PostgreSQL database using it" \
  --conversation-id "learning-session-1"
# Agent knows "it" refers to StatefulSet
```

## Example 3: Knowledge Base Ingestion

```bash
# Manually trigger knowledge base sync
kubectl annotate knowledgebase k8s-runbooks-repo \
  aof.agenticops.org/sync=now

# Check sync status
kubectl get knowledgebase k8s-runbooks-repo -o yaml

# Output:
# status:
#   ready: true
#   stats:
#     totalDocuments: 234
#     totalChunks: 1567
#     lastSyncTime: "2024-01-15T10:30:00Z"
#     lastSyncStatus: success
#   sourceStatus:
#     - name: sre-runbooks
#       status: ready
#       documentsIndexed: 234
#       lastSync: "2024-01-15T10:30:00Z"
```

## Example 4: Multi-Agent Fleet with Shared Memory

```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: AgentFleet
metadata:
  name: incident-response-team
  namespace: sre-platform
spec:
  replicas: 3

  # Shared memory for coordination
  sharedMemory:
    ref: incident-response-memory

    conversational:
      enabled: true
      shared: true  # All agents see same conversation

    rag:
      enabled: true
      topK: 5

  template:
    spec:
      provider: anthropic
      model: claude-3-5-sonnet-20241022

      memory:
        conversational:
          enabled: true
          maxMessages: 20

        rag:
          inheritFromFleet: true

      systemPrompt: |
        You are part of an incident response team. You can see all team
        communications in the shared memory. Coordinate with other agents.
```

**Usage:**
```bash
# Agent 1 processes incident
aof agent query incident-response-team-0 \
  --message "Investigating database connection timeout" \
  --fleet-id "incident-response-team"

# Agent 2 sees Agent 1's message in shared memory
aof agent query incident-response-team-1 \
  --message "What are others working on?" \
  --fleet-id "incident-response-team"
# Response: "Agent 1 is investigating database connection timeout..."

# Agent 3 adds context
aof agent query incident-response-team-2 \
  --message "I found similar issue in past incidents. Checking connection pool settings." \
  --fleet-id "incident-response-team"
```

## Example 5: RAG with Filtering

```bash
# Query with metadata filtering
aof agent query k8s-expert \
  --message "How do I debug a failing ingress?" \
  --conversation-id "debug-session-1" \
  --rag-filter '{"metadata.category": ["networking", "ingress"], "metadata.severity": ["critical", "high"]}'

# The agent will only retrieve runbooks matching the filter
```

## Example 6: Context from Multiple Sources

```yaml
# Agent configuration showing multi-source context
spec:
  context:
    static:
      - name: policies
        source:
          configMapRef:
            name: company-policies

    dynamic:
      - name: incidents
        source:
          type: HTTP
          url: https://api.pagerduty.com/incidents

      - name: cluster-state
        source:
          type: K8sAPI
          resource: nodes

    rag:
      - name: runbooks
        memoryRef: k8s-runbook-memory
        query: "{{.userMessage}}"
        topK: 3
```

**Agent receives context from:**
1. Company policies (static ConfigMap)
2. Current PagerDuty incidents (dynamic HTTP)
3. K8s cluster node status (dynamic K8s API)
4. Relevant runbooks (RAG semantic search)

## Example 7: Workflow with Context Passing

```yaml
apiVersion: aof.agenticops.org/v1alpha1
kind: AgentFlow
metadata:
  name: incident-response-workflow
spec:
  steps:
    - name: detect
      agentRef: detector
      context:
        rag:
          - name: detection-patterns
            memoryRef: incident-memory
            query: "{{.input.alert}}"
      output:
        to: flowContext.detection

    - name: analyze
      agentRef: analyzer
      dependsOn: [detect]
      context:
        inherit: true  # Gets detection results
        dynamic:
          - name: detection-results
            source:
              type: FlowContext
              path: detection
        rag:
          - name: similar-incidents
            memoryRef: incident-memory
            query: "{{.flowContext.detection.issue_type}}"
      output:
        to: flowContext.analysis

    - name: remediate
      agentRef: remediator
      dependsOn: [analyze]
      context:
        inherit: true
        rag:
          - name: remediation-steps
            memoryRef: incident-memory
            query: "{{.flowContext.analysis.root_cause}}"
            filter:
              metadata.category: remediation
```

## Example 8: Observability and Debugging

```bash
# Check memory statistics
kubectl get memory k8s-runbook-memory -o jsonpath='{.status.stats}'
# Output: {"totalEntries":1567,"totalSize":"45.2MB","lastSyncTime":"..."}

# View memory metrics in Prometheus
curl -s 'http://prometheus:9090/api/v1/query?query=aof_memory_entries_total{memory="k8s-runbook-memory"}'

# Check vector store search performance
curl -s 'http://prometheus:9090/api/v1/query?query=aof_vector_search_duration_seconds{memory="k8s-runbook-memory"}'

# Debug RAG retrieval
aof agent query k8s-expert \
  --message "StatefulSet troubleshooting" \
  --debug \
  --show-retrieved-docs

# Output includes:
# Retrieved Documents:
# 1. [Score: 0.89] "Troubleshooting StatefulSet Pod Failures" (runbooks/k8s/statefulset.md)
# 2. [Score: 0.84] "StatefulSet Persistent Volume Issues" (runbooks/storage/pv-issues.md)
# 3. [Score: 0.78] "Kubernetes StatefulSet Best Practices" (docs/k8s/statefulsets.md)
```

## Example 9: Programmatic API Usage

```rust
use aof_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize client
    let client = AgentClient::new("k8s-expert", "sre-platform").await?;

    // Query with conversational memory
    let response = client
        .query("How do I debug CrashLoopBackOff?")
        .conversation_id("incident-2024-001")
        .with_rag(true)
        .send()
        .await?;

    println!("Response: {}", response.content);

    // View retrieved context
    for doc in response.retrieved_docs {
        println!("Source: {} (score: {})", doc.metadata.source, doc.score);
    }

    // Continue conversation
    let follow_up = client
        .query("What if the pod has an OOMKilled exit code?")
        .conversation_id("incident-2024-001")  // Same conversation
        .send()
        .await?;

    println!("Follow-up: {}", follow_up.content);

    Ok(())
}
```

## Performance Benchmarks

### RAG Retrieval Performance

```
Benchmark: 1000 queries with different top-k values

Top-K=3:  Avg: 45ms  P95: 78ms  P99: 120ms
Top-K=5:  Avg: 52ms  P95: 89ms  P99: 145ms
Top-K=10: Avg: 68ms  P95: 115ms P99: 190ms

With Reranking:
Top-K=3:  Avg: 180ms P95: 245ms P99: 380ms
Top-K=5:  Avg: 210ms P95: 289ms P99: 450ms
```

### Memory Backend Performance

```
Redis (conversational memory):
- Write: 0.8ms avg
- Read: 0.6ms avg
- Throughput: 10,000 ops/sec

Qdrant (vector search):
- Index: 100 docs/sec
- Search (top-5): 25ms avg
- Search (top-10): 35ms avg
```

### Knowledge Base Ingestion

```
GitHub Repository (500 markdown files):
- Fetch: 45 seconds
- Chunk: 12 seconds
- Embed: 180 seconds (batch size 100)
- Index: 30 seconds
- Total: ~4.5 minutes

Incremental sync (10 changed files):
- Total: ~30 seconds
```

## Best Practices

1. **Memory Configuration:**
   - Use Redis for conversational memory (fast access)
   - Use Qdrant/Pinecone for semantic memory (optimized vector search)
   - Set appropriate TTLs to manage storage costs

2. **RAG Optimization:**
   - Start with top-k=5, adjust based on needs
   - Enable reranking for critical applications
   - Use metadata filtering to improve relevance

3. **Knowledge Base Management:**
   - Sync frequently updated sources more often
   - Use semantic chunking for technical documentation
   - Enable webhooks for real-time updates

4. **Context Management:**
   - Keep static contexts small and focused
   - Cache dynamic contexts with appropriate refresh intervals
   - Use RAG for large knowledge bases instead of static contexts

5. **Observability:**
   - Monitor RAG retrieval latency
   - Track memory usage growth
   - Alert on knowledge base sync failures
   - Review retrieved documents for relevance
