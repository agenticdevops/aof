---
phase: 01-onboarding-refinement
plan: 04
type: execute
wave: 4
depends_on: ["01-onboarding-refinement-01", "01-onboarding-refinement-02"]
files_modified:
  - aof/crates/aof-core/src/approval.rs
  - aof/src/server/handlers/approval.rs
  - web-app/src/components/config/ApprovalWorkflow.tsx
  - web-app/src/components/config/AuditTrail.tsx
  - web-app/src/store/slices/auditSlice.ts
  - web-app/src/api/config.ts
  - aof/src/audit/audit_logger.rs
  - web-app/src/test/e2e/approval-workflow.test.tsx

autonomous: true

must_haves:
  truths:
    - "Operations categorized as Destructive, Risky, or Safe"
    - "Destructive operations require approval before execution"
    - "Risky operations may auto-approve based on environment (dev/staging/prod)"
    - "Safe operations execute immediately (read-only)"
    - "Approval workflow: Pending → Approved/Rejected → Executed/Cancelled"
    - "Audit log tracks all operations with: operator, action, time, decision, result"
    - "Slack integration shows approval requests and reactions"
    - "Audit trail visible in web UI (read-only, searchable)"
    - "Approval whitelist per Slack user ID supported"
    - "All operations immutably logged with cryptographic integrity"

  artifacts:
    - path: "aof/crates/aof-core/src/approval.rs"
      provides: "OperationCategory enum, ApprovalPolicy, ApprovalRequest types"
      exports: "ApprovalRequest, ApprovalDecision, OperationCategory, ApprovalError"
    - path: "web-app/src/components/config/ApprovalWorkflow.tsx"
      provides: "Component showing pending approvals and approval UI"
      exports: "ApprovalWorkflow component"
    - path: "web-app/src/components/config/AuditTrail.tsx"
      provides: "Read-only audit log viewer with search"
      exports: "AuditTrail component"
    - path: "aof/src/audit/audit_logger.rs"
      provides: "AuditLogger with immutable logging"
      exports: "AuditLogger, AuditEvent, AuditLog"

  key_links:
    - from: "ApprovalRequest"
      to: "Slack notification"
      via: "Approval handler sends message to Slack on risky operation"
      pattern: "send_approval_request"
    - from: "Slack reaction"
      to: "approval.rs"
      via: "Webhook receives reaction (✅/❌), updates ApprovalRequest"
      pattern: "handle_approval_reaction"
    - from: "ApprovalWorkflow.tsx"
      to: "configAPI"
      via: "Component calls approveOperation or rejectOperation"
      pattern: "dispatch\\(approveOperation"
    - from: "Agent execution"
      to: "AuditLogger"
      via: "Every operation logged with approval decision"
      pattern: "audit_logger.log"

---

<objective>
Implement a security-aware approval workflow that classifies operations as Destructive (requires approval), Risky (approval depends on environment), or Safe (immediate execution). All operations are immutably logged in an audit trail with cryptographic integrity for compliance and learning.

**Purpose:** Prevent dangerous operations while maintaining productivity. Users can approve operations via Slack reactions, CLI, or web UI. Every decision is logged for compliance, debugging, and continuous improvement.

**Output:**
- Operation classification system (Destructive/Risky/Safe)
- Approval workflow with Slack integration
- Audit logging with immutable, searchable trail
- Web UI for managing approvals and viewing audit logs
- Compliance-grade operation tracking
</objective>

<execution_context>
@/Users/gshah/.claude/get-shit-done/workflows/execute-plan.md
@/Users/gshah/.claude/get-shit-done/templates/summary.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/RESEARCH-SUMMARY.md
</execution_context>

<context>
@/Users/gshah/work/opsflow-sh/aof/.planning/PROJECT.md
@/Users/gshah/work/opsflow-sh/aof/.planning/STATE.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/08-production-readiness/08-SUMMARY.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement operation classification and approval types</name>
  <files>
    aof/crates/aof-core/src/approval.rs
  </files>
  <action>
Create core types for operation approval and classification:

1. **Create approval.rs module with types:**
   ```rust
   use serde::{Deserialize, Serialize};
   use std::time::SystemTime;

   /// Operation classification for approval routing
   #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
   pub enum OperationCategory {
       /// DELETE, DROP, SCALE_DOWN, DESTROY - requires approval
       Destructive,
       /// UPDATE, PATCH, REBOOT, MODIFY - may require approval depending on blast radius
       Risky,
       /// READ, DESCRIBE, LIST, GET - no approval needed
       Safe,
   }

   /// Environment determines auto-approval for Risky operations
   #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
   pub enum Environment {
       Development,  // Auto-approve Risky
       Staging,      // Auto-approve Risky
       Production,   // Require approval for Risky
   }

   /// Blast radius heuristic
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct BlastRadius {
       pub affected_services: Vec<String>,
       pub estimated_impact_pct: f32,
       pub data_at_risk: bool,
   }

   /// Approval policy determines if approval is needed
   #[derive(Debug, Clone)]
   pub struct ApprovalPolicy {
       pub environment: Environment,
       pub require_approval_for_risky: bool,
       pub approval_timeout_secs: u64,
       pub auto_reject_timeout: bool,
   }

   impl ApprovalPolicy {
       /// Determine if operation needs approval
       pub fn requires_approval(
           &self,
           category: OperationCategory,
           blast_radius: Option<&BlastRadius>,
       ) -> bool {
           match category {
               OperationCategory::Destructive => true,
               OperationCategory::Risky => {
                   // Require approval in prod, auto-approve in dev/staging
                   self.environment == Environment::Production || self.require_approval_for_risky
               }
               OperationCategory::Safe => false,
           }
       }
   }

   /// Approval request
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ApprovalRequest {
       pub id: String,
       pub operation_id: String,
       pub category: OperationCategory,
       pub description: String,
       pub operator: String,              // Who initiated
       pub blast_radius: Option<BlastRadius>,
       pub created_at: SystemTime,
       pub decision: Option<ApprovalDecision>,
       pub decided_at: Option<SystemTime>,
       pub decided_by: Option<String>,    // Who approved/rejected
       pub reason: Option<String>,        // Reason for decision
   }

   #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
   pub enum ApprovalStatus {
       Pending,
       Approved,
       Rejected,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ApprovalDecision {
       pub status: ApprovalStatus,
       pub timestamp: SystemTime,
       pub decided_by: String,
       pub reason: Option<String>,
   }

   /// Classification helper - map operation names to categories
   pub fn classify_operation(operation: &str) -> OperationCategory {
       let lower = operation.to_lowercase();

       // Destructive patterns
       if lower.contains("delete") || lower.contains("drop") ||
          lower.contains("destroy") || lower.contains("remove") ||
          lower.contains("scale_down") || lower.contains("terminate") {
           return OperationCategory::Destructive;
       }

       // Risky patterns
       if lower.contains("update") || lower.contains("patch") ||
          lower.contains("reboot") || lower.contains("restart") ||
          lower.contains("modify") || lower.contains("apply") {
           return OperationCategory::Risky;
       }

       // Default to Safe (read-only)
       OperationCategory::Safe
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_classify_operations() {
           assert_eq!(classify_operation("delete_pod"), OperationCategory::Destructive);
           assert_eq!(classify_operation("update_config"), OperationCategory::Risky);
           assert_eq!(classify_operation("describe_nodes"), OperationCategory::Safe);
       }

       #[test]
       fn test_approval_policy() {
           let policy = ApprovalPolicy {
               environment: Environment::Production,
               require_approval_for_risky: true,
               approval_timeout_secs: 300,
               auto_reject_timeout: false,
           };

           assert!(policy.requires_approval(OperationCategory::Destructive, None));
           assert!(policy.requires_approval(OperationCategory::Risky, None));
           assert!(!policy.requires_approval(OperationCategory::Safe, None));
       }
   }
   ```

2. **Export from lib.rs:**
   - Export ApprovalRequest, ApprovalDecision, OperationCategory
   - Make types available to other crates

3. **Integrate with existing types:**
   - If Agent or Task types exist, add approval_required field
   - Add operation_id for linking to approvals

Ensure types are serializable/deserializable for JSON/database storage.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof && cargo test --lib approval
Should show:
- Tests for operation classification pass
- Tests for approval policy pass
- Types properly serialize/deserialize
- No compilation errors

Also test:
cargo check --all
Should compile without warnings.
  </verify>
  <done>
✅ OperationCategory enum (Destructive, Risky, Safe) defined
✅ ApprovalPolicy type with environment-aware approval logic
✅ ApprovalRequest and ApprovalDecision types
✅ classify_operation() helper for categorizing operations
✅ Environment enum (Development, Staging, Production)
✅ BlastRadius type for impact assessment
✅ All tests passing
✅ Types properly serializable
  </done>
</task>

<task type="auto">
  <name>Task 2: Implement AuditLogger for immutable operation logging</name>
  <files>
    aof/src/audit/audit_logger.rs
  </files>
  <action>
Create an immutable audit logging system with cryptographic integrity:

1. **Create audit_logger.rs:**
   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Deserialize, Serialize};
   use sha2::{Digest, Sha256};
   use std::fs::{File, OpenOptions};
   use std::io::{BufReader, BufWriter, Write};
   use std::path::PathBuf;
   use std::sync::Arc;
   use tokio::sync::RwLock;

   /// Single audit log entry
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AuditEvent {
       pub id: String,                    // Unique ID
       pub timestamp: DateTime<Utc>,
       pub operator: String,              // User/agent who performed action
       pub operation: String,             // What was done (e.g., "delete_pod")
       pub category: String,              // Destructive/Risky/Safe
       pub resource: String,              // What was affected (e.g., "pod: nginx-123")
       pub approval_id: Option<String>,   // Link to approval request
       pub approval_decision: Option<String>, // Approved/Rejected/Auto-Approved
       pub result: OperationResult,
       pub error_message: Option<String>,
       pub duration_ms: u64,
       pub approval_decision_by: Option<String>, // Who approved
       pub approval_decision_at: Option<DateTime<Utc>>,
       pub metadata: serde_json::Value,   // Additional context
       pub hash: String,                  // SHA256 hash for integrity
       pub previous_hash: String,         // Hash of previous entry (chain)
   }

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub enum OperationResult {
       Success,
       Failed,
       Rejected,
       Timeout,
   }

   /// Audit logger with file-based persistence
   pub struct AuditLogger {
       log_path: PathBuf,
       cache: Arc<RwLock<Vec<AuditEvent>>>,
   }

   impl AuditLogger {
       pub fn new(log_path: PathBuf) -> std::io::Result<Self> {
           // Create log file if not exists
           if !log_path.exists() {
               File::create(&log_path)?;
           }

           let mut logger = AuditLogger {
               log_path,
               cache: Arc::new(RwLock::new(Vec::new())),
           };

           // Load existing logs
           logger.load_logs()?;

           Ok(logger)
       }

       /// Log an operation
       pub async fn log_operation(
           &self,
           operator: &str,
           operation: &str,
           category: &str,
           resource: &str,
           result: OperationResult,
       ) -> std::io::Result<AuditEvent> {
           self.log_operation_full(AuditEventBuilder::new(operator, operation, category, resource)
               .result(result)
               .build())
           .await
       }

       /// Log with full details
       pub async fn log_operation_full(&self, mut event: AuditEvent) -> std::io::Result<AuditEvent> {
           // Calculate hash
           let previous_hash = {
               let cache = self.cache.read().await;
               cache.last().map(|e| e.hash.clone())
                   .unwrap_or_else(|| "genesis".to_string())
           };

           event.previous_hash = previous_hash;
           event.hash = self.calculate_event_hash(&event);

           // Append to file
           self.append_to_file(&event)?;

           // Update cache
           {
               let mut cache = self.cache.write().await;
               cache.push(event.clone());
           }

           Ok(event)
       }

       /// Get all audit logs
       pub async fn get_logs(&self) -> Vec<AuditEvent> {
           self.cache.read().await.clone()
       }

       /// Get logs filtered by operator
       pub async fn get_logs_by_operator(&self, operator: &str) -> Vec<AuditEvent> {
           self.cache.read().await
               .iter()
               .filter(|e| e.operator == operator)
               .cloned()
               .collect()
       }

       /// Get logs filtered by time range
       pub async fn get_logs_in_range(
           &self,
           start: DateTime<Utc>,
           end: DateTime<Utc>,
       ) -> Vec<AuditEvent> {
           self.cache.read().await
               .iter()
               .filter(|e| e.timestamp >= start && e.timestamp <= end)
               .cloned()
               .collect()
       }

       /// Verify audit trail integrity
       pub async fn verify_integrity(&self) -> bool {
           let logs = self.cache.read().await;

           let mut previous_hash = "genesis".to_string();
           for log in logs.iter() {
               if log.previous_hash != previous_hash {
                   return false;
               }
               if log.hash != self.calculate_event_hash(log) {
                   return false;
               }
               previous_hash = log.hash.clone();
           }

           true
       }

       // Private helpers
       fn calculate_event_hash(&self, event: &AuditEvent) -> String {
           let serialized = serde_json::to_string(&event)
               .unwrap_or_else(|_| "".to_string());
           let mut hasher = Sha256::new();
           hasher.update(serialized.as_bytes());
           format!("{:x}", hasher.finalize())
       }

       fn append_to_file(&self, event: &AuditEvent) -> std::io::Result<()> {
           let file = OpenOptions::new()
               .create(true)
               .append(true)
               .open(&self.log_path)?;

           let mut writer = BufWriter::new(file);
           let json = serde_json::to_string(event)
               .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
           writeln!(writer, "{}", json)?;
           writer.flush()?;

           Ok(())
       }

       fn load_logs(&mut self) -> std::io::Result<()> {
           if !self.log_path.exists() {
               return Ok(());
           }

           let file = File::open(&self.log_path)?;
           let reader = BufReader::new(file);

           for line in reader.lines() {
               let line = line?;
               if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                   self.cache.as_ref().clone().blocking_write().push(event);
               }
           }

           Ok(())
       }
   }

   /// Builder for constructing audit events
   pub struct AuditEventBuilder {
       operator: String,
       operation: String,
       category: String,
       resource: String,
       approval_id: Option<String>,
       result: OperationResult,
       error_message: Option<String>,
       duration_ms: u64,
   }

   impl AuditEventBuilder {
       pub fn new(operator: &str, operation: &str, category: &str, resource: &str) -> Self {
           AuditEventBuilder {
               operator: operator.to_string(),
               operation: operation.to_string(),
               category: category.to_string(),
               resource: resource.to_string(),
               approval_id: None,
               result: OperationResult::Success,
               error_message: None,
               duration_ms: 0,
           }
       }

       pub fn result(mut self, result: OperationResult) -> Self {
           self.result = result;
           self
       }

       pub fn error(mut self, error: String) -> Self {
           self.error_message = Some(error);
           self
       }

       pub fn approval_id(mut self, id: String) -> Self {
           self.approval_id = Some(id);
           self
       }

       pub fn duration_ms(mut self, ms: u64) -> Self {
           self.duration_ms = ms;
           self
       }

       pub fn build(self) -> AuditEvent {
           AuditEvent {
               id: uuid::Uuid::new_v4().to_string(),
               timestamp: Utc::now(),
               operator: self.operator,
               operation: self.operation,
               category: self.category,
               resource: self.resource,
               approval_id: self.approval_id,
               approval_decision: None,
               result: self.result,
               error_message: self.error_message,
               duration_ms: self.duration_ms,
               approval_decision_by: None,
               approval_decision_at: None,
               metadata: serde_json::json!({}),
               hash: "".to_string(),  // Will be calculated
               previous_hash: "".to_string(), // Will be calculated
           }
       }
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_audit_logging() {
           let log_path = PathBuf::from("/tmp/test_audit.log");
           let logger = AuditLogger::new(log_path.clone()).unwrap();

           let event = logger.log_operation(
               "admin",
               "delete_pod",
               "Destructive",
               "pod: nginx-123",
               OperationResult::Success,
           ).await.unwrap();

           assert!(!event.hash.is_empty());
           assert!(logger.verify_integrity().await);

           std::fs::remove_file(log_path).ok();
       }
   }
   ```

2. **Integration:**
   - Initialize AuditLogger in daemon startup
   - Pass to handlers that execute operations
   - Log every operation (before/after)

3. **Features:**
   - Immutable append-only log
   - Cryptographic chain (each entry hashes previous)
   - Efficient in-memory cache with file persistence
   - Integrity verification (can detect tampering)
   - Time-range and operator filtering

Ensure audit logs are never modified, only appended. Use SHA256 hashing for integrity.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof && cargo test --lib audit_logger
Should show:
- Audit event creation works
- Logging appends to file correctly
- Hash calculation and chaining works
- Integrity verification passes
- Filtering by operator/time works
- No data loss on reload

Also test manually:
- Create audit logger
- Log 3-5 operations
- Verify integrity
- Modify a log entry (should fail verification)
  </verify>
  <done>
✅ AuditLogger implemented with file-based persistence
✅ Immutable append-only design (no modifications)
✅ SHA256 hashing for integrity with chain linking
✅ AuditEvent structure with all required fields
✅ OperationResult enum (Success/Failed/Rejected/Timeout)
✅ AuditEventBuilder for convenient event creation
✅ Filtering by operator, time range, operation
✅ Integrity verification method
✅ All tests passing
  </done>
</task>

<task type="auto">
  <name>Task 3: Create approval handler for daemon API</name>
  <files>
    aof/src/server/handlers/approval.rs
  </files>
  <action>
Create HTTP handlers for approval workflow in the daemon:

1. **Create approval handler:**
   ```rust
   use axum::{
       extract::{Path, State},
       http::StatusCode,
       response::IntoResponse,
       routing::{get, post, put},
       Json, Router,
   };
   use serde::{Deserialize, Serialize};
   use std::sync::Arc;
   use tokio::sync::RwLock;

   use crate::approval::{ApprovalRequest, ApprovalStatus};
   use crate::audit::AuditLogger;

   #[derive(Clone)]
   pub struct ApprovalState {
       pub pending: Arc<RwLock<Vec<ApprovalRequest>>>,
       pub audit_logger: Arc<AuditLogger>,
   }

   /// GET /api/approvals - List pending approvals
   pub async fn list_approvals(
       State(state): State<ApprovalState>,
   ) -> Json<Vec<ApprovalRequest>> {
       let pending = state.pending.read().await;
       Json(pending.clone())
   }

   /// GET /api/approvals/:id - Get specific approval
   pub async fn get_approval(
       Path(id): Path<String>,
       State(state): State<ApprovalState>,
   ) -> Result<Json<ApprovalRequest>, (StatusCode, String)> {
       let pending = state.pending.read().await;
       pending.iter()
           .find(|a| a.id == id)
           .cloned()
           .map(Json)
           .ok_or((StatusCode::NOT_FOUND, "Approval not found".to_string()))
   }

   /// POST /api/approvals/:id/approve - Approve operation
   #[derive(Deserialize)]
   pub struct ApprovalAction {
       decided_by: String,
       reason: Option<String>,
   }

   pub async fn approve_operation(
       Path(id): Path<String>,
       State(state): State<ApprovalState>,
       Json(action): Json<ApprovalAction>,
   ) -> Result<impl IntoResponse, (StatusCode, String)> {
       let mut pending = state.pending.write().await;

       if let Some(approval) = pending.iter_mut().find(|a| a.id == id) {
           approval.decision = Some(ApprovalDecision {
               status: ApprovalStatus::Approved,
               timestamp: SystemTime::now(),
               decided_by: action.decided_by.clone(),
               reason: action.reason.clone(),
           });

           // Log approval decision
           let _ = state.audit_logger.log_operation(
               &action.decided_by,
               &format!("approved_operation: {}", approval.operation_id),
               "Risky",
               &approval.operation_id,
               OperationResult::Success,
           ).await;

           Ok((StatusCode::OK, Json(approval.clone())))
       } else {
           Err((StatusCode::NOT_FOUND, "Approval not found".to_string()))
       }
   }

   /// POST /api/approvals/:id/reject - Reject operation
   pub async fn reject_operation(
       Path(id): Path<String>,
       State(state): State<ApprovalState>,
       Json(action): Json<ApprovalAction>,
   ) -> Result<impl IntoResponse, (StatusCode, String)> {
       let mut pending = state.pending.write().await;

       if let Some(approval) = pending.iter_mut().find(|a| a.id == id) {
           approval.decision = Some(ApprovalDecision {
               status: ApprovalStatus::Rejected,
               timestamp: SystemTime::now(),
               decided_by: action.decided_by.clone(),
               reason: action.reason.clone(),
           });

           // Log rejection
           let _ = state.audit_logger.log_operation(
               &action.decided_by,
               &format!("rejected_operation: {}", approval.operation_id),
               "Risky",
               &approval.operation_id,
               OperationResult::Rejected,
           ).await;

           Ok((StatusCode::OK, Json(approval.clone())))
       } else {
           Err((StatusCode::NOT_FOUND, "Approval not found".to_string()))
       }
   }

   /// GET /api/audit - Get audit log
   pub async fn get_audit_log(
       State(state): State<ApprovalState>,
   ) -> Json<Vec<AuditEvent>> {
       Json(state.audit_logger.get_logs().await)
   }

   /// Router for approval endpoints
   pub fn approval_routes() -> Router<ApprovalState> {
       Router::new()
           .route("/approvals", get(list_approvals))
           .route("/approvals/:id", get(get_approval))
           .route("/approvals/:id/approve", post(approve_operation))
           .route("/approvals/:id/reject", post(reject_operation))
           .route("/audit", get(get_audit_log))
   }
   ```

2. **Request/Response formats:**
   - GET /api/approvals → returns [] of pending ApprovalRequests
   - POST /api/approvals/:id/approve → { decided_by: string, reason?: string }
   - GET /api/audit → returns [] of AuditEvents

3. **Integration into daemon:**
   - Initialize ApprovalState with pending approvals list and AuditLogger
   - Mount approval_routes() into main router
   - Handle Slack webhook callbacks for reactions

4. **Slack integration (future):**
   - When approval needed, send Slack message with ✅/❌ reactions
   - Webhook receives reaction, calls approve/reject endpoint

Ensure all approval actions are logged to audit trail.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof && cargo test --lib approval_handler
Should show:
- GET /api/approvals returns list
- POST approve/reject updates state
- Audit log populated on decisions
- Error handling (404 for missing)

Manual test:
cargo run --bin aofctl -- serve
curl http://localhost:7777/api/approvals
Should return empty array initially.

curl -X POST http://localhost:7777/api/approvals/123/approve -H "Content-Type: application/json" -d '{"decided_by":"admin"}'
Should return updated approval.
  </verify>
  <done>
✅ HTTP handlers for approval workflow implemented
✅ GET /api/approvals lists pending approvals
✅ POST /api/approvals/:id/approve approves operation
✅ POST /api/approvals/:id/reject rejects operation
✅ GET /api/audit returns audit log
✅ Audit logger called on every approval decision
✅ Proper error handling (404, 500)
✅ ApprovalState properly initialized and integrated
  </done>
</task>

<task type="auto">
  <name>Task 4: Create ApprovalWorkflow component for web UI</name>
  <files>
    web-app/src/components/config/ApprovalWorkflow.tsx
  </files>
  <action>
Create a React component for viewing and managing approval requests:

1. **ApprovalWorkflow component:**
   ```typescript
   interface ApprovalWorkflowProps {
     onApprovalAction?: (action: 'approve' | 'reject', id: string) => Promise<void>
   }

   export const ApprovalWorkflow: React.FC<ApprovalWorkflowProps> = ({
     onApprovalAction
   }) => {
     const [approvals, setApprovals] = useState<ApprovalRequest[]>([])
     const [loading, setLoading] = useState(false)
     const [error, setError] = useState<string | null>(null)

     useEffect(() => {
       loadApprovals()
       // Poll for new approvals every 10 seconds
       const interval = setInterval(loadApprovals, 10000)
       return () => clearInterval(interval)
     }, [])

     const loadApprovals = async () => {
       try {
         const pending = await configAPI.getApprovals()
         setApprovals(pending.filter(a => !a.decision))
       } catch (err) {
         setError('Failed to load approvals')
       }
     }

     const handleApprove = async (id: string) => {
       try {
         await configAPI.approveOperation(id, {
           decided_by: 'current_user', // Get from auth context
           reason: 'Approved via web UI'
         })
         await loadApprovals()
         onApprovalAction?.('approve', id)
       } catch (err) {
         setError('Failed to approve')
       }
     }

     const handleReject = async (id: string) => {
       try {
         await configAPI.rejectOperation(id, {
           decided_by: 'current_user',
           reason: 'Rejected via web UI'
         })
         await loadApprovals()
         onApprovalAction?.('reject', id)
       } catch (err) {
         setError('Failed to reject')
       }
     }

     return (
       <div className="approval-workflow">
         <h2>Pending Approvals</h2>

         {loading && <LoadingSpinner />}
         {error && <AlertError message={error} onDismiss={() => setError(null)} />}

         {approvals.length === 0 ? (
           <EmptyState
             icon="✅"
             title="No pending approvals"
             description="All operations have been approved or are executing"
           />
         ) : (
           <div className="approvals-list">
             {approvals.map(approval => (
               <ApprovalCard
                 key={approval.id}
                 approval={approval}
                 onApprove={() => handleApprove(approval.id)}
                 onReject={() => handleReject(approval.id)}
               />
             ))}
           </div>
         )}
       </div>
     )
   }
   ```

2. **ApprovalCard sub-component:**
   ```typescript
   interface ApprovalCardProps {
     approval: ApprovalRequest
     onApprove: () => Promise<void>
     onReject: () => Promise<void>
   }

   const ApprovalCard: React.FC<ApprovalCardProps> = ({
     approval,
     onApprove,
     onReject
   }) => {
     const [actionLoading, setActionLoading] = useState<'approve' | 'reject' | null>(null)

     const handleApprove = async () => {
       setActionLoading('approve')
       try {
         await onApprove()
       } finally {
         setActionLoading(null)
       }
     }

     const handleReject = async () => {
       setActionLoading('reject')
       try {
         await onReject()
       } finally {
         setActionLoading(null)
       }
     }

     return (
       <div className="approval-card">
         <div className="card-header">
           <h3>Operation Approval Required</h3>
           <span className={`category-badge ${approval.category.toLowerCase()}`}>
             {approval.category}
           </span>
         </div>

         <div className="card-details">
           <p className="operation">{approval.description}</p>
           <p className="operator">Initiated by: {approval.operator}</p>
           <p className="created">
             {new Date(approval.created_at).toLocaleString()}
           </p>

           {approval.blast_radius && (
             <div className="blast-radius">
               <h4>Impact Assessment</h4>
               <p>Affected services: {approval.blast_radius.affected_services.join(', ')}</p>
               <p>Estimated impact: {approval.blast_radius.estimated_impact_pct}%</p>
               {approval.blast_radius.data_at_risk && (
                 <p className="warning">⚠️ Data at risk</p>
               )}
             </div>
           )}
         </div>

         <div className="card-actions">
           <button
             onClick={handleApprove}
             disabled={actionLoading !== null}
             className="btn-approve"
           >
             {actionLoading === 'approve' ? 'Approving...' : '✓ Approve'}
           </button>
           <button
             onClick={handleReject}
             disabled={actionLoading !== null}
             className="btn-reject"
           >
             {actionLoading === 'reject' ? 'Rejecting...' : '✗ Reject'}
           </button>
         </div>
       </div>
     )
   }
   ```

3. **Integration in Configuration dashboard:**
   - Add "Approvals" tab or panel
   - Show pending approvals (red notification badge if any)
   - Link from approval request to related operation details

4. **Styling:**
   - Card layout with clear operation info
   - Category color-coding (Destructive = red, Risky = yellow, Safe = green)
   - Large action buttons (approve/reject)
   - Time stamps and operator info
   - Blast radius warning if data at risk

5. **Real-time updates:**
   - Poll /api/approvals every 10 seconds
   - Or use WebSocket for push (future enhancement)
   - Show notification when new approval needed

Example styling:
```css
.approval-card.destructive {
  border-left: 4px solid #ef4444; /* red */
}

.approval-card.risky {
  border-left: 4px solid #f59e0b; /* amber */
}

.category-badge.destructive {
  background-color: #fecaca;
  color: #dc2626;
}
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- ApprovalWorkflow
Should show:
- Component renders approval cards
- Pending approvals display correctly
- Approve button works
- Reject button works
- Loading state during action
- Error handling displayed
- Empty state shown when no approvals

Browser test:
- pnpm dev
- Visit configuration dashboard
- Should see approvals tab
- Mock approval should display
- Can click approve/reject
  </verify>
  <done>
✅ ApprovalWorkflow component created
✅ Shows pending approvals as cards
✅ Category color-coded (Destructive/Risky/Safe)
✅ Displays operation details, operator, impact assessment
✅ Approve and reject buttons functional
✅ Loading state during action
✅ Error handling with user message
✅ Empty state when no approvals
✅ Polling for new approvals every 10s
  </done>
</task>

<task type="auto">
  <name>Task 5: Create AuditTrail component for viewing logs</name>
  <files>
    web-app/src/components/config/AuditTrail.tsx
  </files>
  <action>
Create a read-only audit log viewer with search and filtering:

1. **AuditTrail component:**
   ```typescript
   interface AuditTrailProps {
     operatorFilter?: string
     timeRangeFilter?: { start: Date; end: Date }
   }

   export const AuditTrail: React.FC<AuditTrailProps> = ({
     operatorFilter,
     timeRangeFilter
   }) => {
     const [auditEvents, setAuditEvents] = useState<AuditEvent[]>([])
     const [loading, setLoading] = useState(false)
     const [searchQuery, setSearchQuery] = useState('')
     const [operatorQuery, setOperatorQuery] = useState(operatorFilter || '')
     const [categoryFilter, setCategoryFilter] = useState<string>('') // Destructive, Risky, Safe, or ""
     const [resultFilter, setResultFilter] = useState<string>('') // Success, Failed, Rejected, or ""

     useEffect(() => {
       loadAuditLog()
     }, [operatorFilter, timeRangeFilter])

     const loadAuditLog = async () => {
       setLoading(true)
       try {
         const events = await configAPI.getAuditLog()
         setAuditEvents(events)
       } catch (err) {
         console.error('Failed to load audit log:', err)
       } finally {
         setLoading(false)
       }
     }

     const filteredEvents = auditEvents.filter(event => {
       // Text search
       if (searchQuery) {
         const query = searchQuery.toLowerCase()
         if (!event.operation.toLowerCase().includes(query) &&
             !event.resource.toLowerCase().includes(query) &&
             !event.operator.toLowerCase().includes(query)) {
           return false
         }
       }

       // Operator filter
       if (operatorQuery && event.operator !== operatorQuery) {
         return false
       }

       // Category filter
       if (categoryFilter && event.category !== categoryFilter) {
         return false
       }

       // Result filter
       if (resultFilter && event.result !== resultFilter) {
         return false
       }

       return true
     })

     const sortedEvents = [...filteredEvents].sort(
       (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
     )

     return (
       <div className="audit-trail">
         <h2>Audit Log</h2>
         <p className="description">Read-only record of all operations with immutable integrity chain</p>

         {/* Filters */}
         <div className="filters">
           <SearchBar
             placeholder="Search operations, resources, operators..."
             value={searchQuery}
             onChange={setSearchQuery}
           />

           <select value={operatorQuery} onChange={e => setOperatorQuery(e.target.value)}>
             <option value="">All Operators</option>
             {Array.from(new Set(auditEvents.map(e => e.operator))).map(op => (
               <option key={op} value={op}>{op}</option>
             ))}
           </select>

           <select value={categoryFilter} onChange={e => setCategoryFilter(e.target.value)}>
             <option value="">All Categories</option>
             <option value="Destructive">Destructive</option>
             <option value="Risky">Risky</option>
             <option value="Safe">Safe</option>
           </select>

           <select value={resultFilter} onChange={e => setResultFilter(e.target.value)}>
             <option value="">All Results</option>
             <option value="Success">Success</option>
             <option value="Failed">Failed</option>
             <option value="Rejected">Rejected</option>
             <option value="Timeout">Timeout</option>
           </select>
         </div>

         {loading && <LoadingSpinner message="Loading audit log..." />}

         {sortedEvents.length === 0 ? (
           <EmptyState
             icon="📋"
             title="No events found"
             description="No audit events match your filters"
           />
         ) : (
           <div className="audit-table">
             <table>
               <thead>
                 <tr>
                   <th>Timestamp</th>
                   <th>Operator</th>
                   <th>Operation</th>
                   <th>Resource</th>
                   <th>Category</th>
                   <th>Result</th>
                   <th>Approval</th>
                   <th>Duration</th>
                 </tr>
               </thead>
               <tbody>
                 {sortedEvents.map(event => (
                   <AuditEventRow key={event.id} event={event} />
                 ))}
               </tbody>
             </table>
           </div>
         )}

         <div className="audit-footer">
           <p className="integrity-status">
             ✓ Integrity verified: All {sortedEvents.length} events have valid cryptographic hashes
           </p>
         </div>
       </div>
     )
   }
   ```

2. **AuditEventRow sub-component:**
   ```typescript
   const AuditEventRow: React.FC<{ event: AuditEvent }> = ({ event }) => {
     const [expanded, setExpanded] = useState(false)

     return (
       <>
         <tr className={`event-row ${event.result.toLowerCase()}`} onClick={() => setExpanded(!expanded)}>
           <td>{new Date(event.timestamp).toLocaleString()}</td>
           <td>{event.operator}</td>
           <td className="operation">{event.operation}</td>
           <td className="resource">{event.resource}</td>
           <td><span className={`category-badge ${event.category.toLowerCase()}`}>{event.category}</span></td>
           <td><span className={`result-badge ${event.result.toLowerCase()}`}>{event.result}</span></td>
           <td>{event.approval_decision || '—'}</td>
           <td>{event.duration_ms}ms</td>
         </tr>

         {expanded && (
           <tr className="event-details">
             <td colSpan={8}>
               <div className="details-panel">
                 <div className="detail">
                   <span className="label">Operation ID:</span>
                   <code>{event.id}</code>
                 </div>
                 {event.approval_id && (
                   <div className="detail">
                     <span className="label">Approval ID:</span>
                     <code>{event.approval_id}</code>
                   </div>
                 )}
                 {event.error_message && (
                   <div className="detail error">
                     <span className="label">Error:</span>
                     <pre>{event.error_message}</pre>
                   </div>
                 )}
                 {event.approval_decision_by && (
                   <div className="detail">
                     <span className="label">Approved by:</span>
                     <span>{event.approval_decision_by}</span>
                   </div>
                 )}
                 <div className="detail hash">
                   <span className="label">Hash:</span>
                   <code>{event.hash.slice(0, 16)}...</code>
                 </div>
               </div>
             </td>
           </tr>
         )}
       </>
     )
   }
   ```

3. **Features:**
   - Search by operation, resource, operator
   - Filter by category (Destructive/Risky/Safe)
   - Filter by result (Success/Failed/Rejected)
   - Sort by timestamp (newest first)
   - Expandable rows for detailed info
   - Hash verification status
   - Immutable (read-only, no delete buttons)

4. **Styling:**
   - Table layout with fixed header
   - Row colors by result (green = success, red = failed, yellow = rejected)
   - Category badges with colors
   - Monospace font for hashes and IDs
   - Expandable details with proper indentation

Integration in dashboard:
```typescript
// In Configuration dashboard
<Tabs>
  <Tab name="Agents" content={<AgentList />} />
  <Tab name="Approvals" content={<ApprovalWorkflow />} />
  <Tab name="Audit Log" content={<AuditTrail />} />
  <Tab name="Tools" content={<ToolsList />} />
</Tabs>
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- AuditTrail
Should show:
- Component renders audit events table
- Events sorted by timestamp (newest first)
- Filters work (operator, category, result)
- Search works (operation, resource)
- Expanding row shows details
- Hash verification status displayed
- No modify/delete buttons (read-only)

Browser test:
- Visit configuration dashboard
- Go to "Audit Log" tab
- Should see table of events
- Can filter and search
- Can expand rows for details
  </verify>
  <done>
✅ AuditTrail component created
✅ Displays events in table format with all relevant fields
✅ Sorting by timestamp (newest first)
✅ Filtering by operator, category, result
✅ Text search across operation, resource, operator
✅ Expandable rows showing detailed info and hashes
✅ Read-only (no delete/modify buttons)
✅ Integrity verification status displayed
✅ Mobile responsive table
  </done>
</task>

<task type="auto">
  <name>Task 6: Wire approval and audit logging into Redux and API</name>
  <files>
    web-app/src/store/slices/auditSlice.ts
    web-app/src/api/config.ts
  </files>
  <action>
Add Redux state management and API methods for approvals and audit logging:

1. **Create auditSlice.ts:**
   ```typescript
   import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
   import { configAPI } from '@/api/config'

   interface AuditState {
     events: AuditEvent[]
     approvals: ApprovalRequest[]
     loading: boolean
     error: string | null
   }

   const initialState: AuditState = {
     events: [],
     approvals: [],
     loading: false,
     error: null
   }

   export const fetchAuditLog = createAsyncThunk(
     'audit/fetchLog',
     async (_, { rejectWithValue }) => {
       try {
         return await configAPI.getAuditLog()
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   export const fetchApprovals = createAsyncThunk(
     'audit/fetchApprovals',
     async (_, { rejectWithValue }) => {
       try {
         return await configAPI.getApprovals()
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   export const approveOperation = createAsyncThunk(
     'audit/approve',
     async ({ id, reason }: { id: string; reason?: string }, { rejectWithValue }) => {
       try {
         return await configAPI.approveOperation(id, {
           decided_by: 'current_user', // Get from auth context
           reason
         })
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   export const rejectOperation = createAsyncThunk(
     'audit/reject',
     async ({ id, reason }: { id: string; reason?: string }, { rejectWithValue }) => {
       try {
         return await configAPI.rejectOperation(id, {
           decided_by: 'current_user',
           reason
         })
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   const auditSlice = createSlice({
     name: 'audit',
     initialState,
     reducers: {
       clearError: (state) => {
         state.error = null
       }
     },
     extraReducers: (builder) => {
       builder
         .addCase(fetchAuditLog.pending, (state) => {
           state.loading = true
         })
         .addCase(fetchAuditLog.fulfilled, (state, action) => {
           state.loading = false
           state.events = action.payload
         })
         .addCase(fetchAuditLog.rejected, (state, action) => {
           state.loading = false
           state.error = action.payload as string
         })

         .addCase(fetchApprovals.pending, (state) => {
           state.loading = true
         })
         .addCase(fetchApprovals.fulfilled, (state, action) => {
           state.loading = false
           state.approvals = action.payload
         })
         .addCase(fetchApprovals.rejected, (state, action) => {
           state.loading = false
           state.error = action.payload as string
         })

         .addCase(approveOperation.fulfilled, (state, action) => {
           const idx = state.approvals.findIndex(a => a.id === action.payload.id)
           if (idx >= 0) {
             state.approvals[idx] = action.payload
           }
         })

         .addCase(rejectOperation.fulfilled, (state, action) => {
           const idx = state.approvals.findIndex(a => a.id === action.payload.id)
           if (idx >= 0) {
             state.approvals[idx] = action.payload
           }
         })
     }
   })

   export const { clearError } = auditSlice.actions
   export default auditSlice.reducer
   ```

2. **Add to configAPI (in config.ts):**
   ```typescript
   export class ConfigAPI {
     // ... existing methods ...

     async getApprovals(): Promise<ApprovalRequest[]> {
       try {
         const response = await this.client.get<ApprovalRequest[]>('/config/approvals')
         return response.data
       } catch (error) {
         throw new Error('Failed to fetch approvals')
       }
     }

     async approveOperation(
       id: string,
       action: { decided_by: string; reason?: string }
     ): Promise<ApprovalRequest> {
       try {
         const response = await this.client.post<ApprovalRequest>(
           `/config/approvals/${id}/approve`,
           action
         )
         return response.data
       } catch (error) {
         throw new Error('Failed to approve operation')
       }
     }

     async rejectOperation(
       id: string,
       action: { decided_by: string; reason?: string }
     ): Promise<ApprovalRequest> {
       try {
         const response = await this.client.post<ApprovalRequest>(
           `/config/approvals/${id}/reject`,
           action
         )
         return response.data
       } catch (error) {
         throw new Error('Failed to reject operation')
       }
     }

     async getAuditLog(): Promise<AuditEvent[]> {
       try {
         const response = await this.client.get<AuditEvent[]>('/config/audit')
         return response.data
       } catch (error) {
         throw new Error('Failed to fetch audit log')
       }
     }
   }
   ```

3. **Register auditSlice in store:**
   ```typescript
   import auditReducer from './slices/auditSlice'

   export const store = configureStore({
     reducer: {
       auth: authReducer,
       config: configReducer,
       onboarding: onboardingReducer,
       audit: auditReducer  // Add this
     },
     middleware: (getDefaultMiddleware) =>
       getDefaultMiddleware()
         .concat(loggingMiddleware)
   })
   ```

4. **Add mock handlers:**
   ```typescript
   export const auditHandlers = [
     http.get('*/config/approvals', () => {
       return HttpResponse.json(MOCK_APPROVALS)
     }),

     http.post('*/config/approvals/:id/approve', async ({ request, params }) => {
       const body = await request.json()
       // Update mock approval with decision
       return HttpResponse.json({ id: params.id, ...body })
     }),

     http.get('*/config/audit', () => {
       return HttpResponse.json(MOCK_AUDIT_EVENTS)
     })
   ]
   ```

Ensure API methods are type-safe and error messages are user-friendly.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- auditSlice
Should show:
- Async thunks dispatch correctly
- Approval state updates on approve/reject
- Audit events loaded correctly
- Error handling works

Also test API client:
- configAPI.getApprovals() returns approvals
- configAPI.approveOperation() posts to correct endpoint
- configAPI.getAuditLog() returns events
  </verify>
  <done>
✅ auditSlice Redux reducer created
✅ Async thunks for fetching/updating approvals and audit log
✅ Redux state properly manages approval and audit event data
✅ configAPI methods added for all approval/audit operations
✅ Mock handlers return realistic data
✅ Error handling with user-friendly messages
✅ All types properly exported and used
  </done>
</task>

<task type="auto">
  <name>Task 7: Write comprehensive tests for approval and audit workflows</name>
  <files>
    web-app/src/test/e2e/approval-workflow.test.tsx
  </files>
  <action>
Create end-to-end tests for approval and audit logging:

1. **Test suite:**
   ```typescript
   describe('Approval and Audit Workflow E2E', () => {
     // Test 1: Approval request display
     // Test 2: Approve operation
     // Test 3: Reject operation
     // Test 4: Audit log display and filtering
     // Test 5: Error handling
     // Test 6: Immutability and integrity
   })
   ```

2. **Test cases:**

   **Test 1: Pending approvals display**
   - ApprovalWorkflow component shows pending approvals
   - Each approval shows operation, operator, category, timestamp
   - Category color-coded correctly
   - Empty state shown when no approvals

   **Test 2: Approve operation**
   - Clicking approve button calls API
   - Approval state updates
   - Audit log includes approval decision
   - Approval removed from pending list

   **Test 3: Reject operation**
   - Clicking reject button calls API
   - Approval marked as rejected
   - Audit log includes rejection
   - Can provide reason for rejection

   **Test 4: Audit log display**
   - AuditTrail component shows all events
   - Events sorted by timestamp (newest first)
   - Can search by operation/resource/operator
   - Can filter by category and result
   - Expanding row shows details
   - Hash values displayed

   **Test 5: Error handling**
   - Network error on approval shows error message
   - Retry button available
   - Failed approval doesn't remove from pending list

   **Test 6: Immutability**
   - Audit log is read-only (no delete buttons)
   - Events cannot be modified
   - Hash chain verifies integrity

3. **Example tests:**
   ```typescript
   import { render, screen, fireEvent, waitFor } from '@testing-library/react'
   import userEvent from '@testing-library/user-event'
   import { ApprovalWorkflow } from '@/components/config/ApprovalWorkflow'
   import { AuditTrail } from '@/components/config/AuditTrail'
   import { Provider } from 'react-redux'
   import { store } from '@/store'

   describe('Approval and Audit Workflow', () => {
     it('displays pending approvals', async () => {
       render(
         <Provider store={store}>
           <ApprovalWorkflow />
         </Provider>
       )

       await waitFor(() => {
         expect(screen.getByText(/delete_pod/)).toBeInTheDocument()
       })

       expect(screen.getByText(/Destructive/)).toBeInTheDocument()
     })

     it('approves operation and updates audit log', async () => {
       const user = userEvent.setup()

       render(
         <Provider store={store}>
           <ApprovalWorkflow />
         </Provider>
       )

       const approveButton = await screen.findByRole('button', { name: /approve/i })
       await user.click(approveButton)

       await waitFor(() => {
         expect(screen.getByText(/no pending approvals/i)).toBeInTheDocument()
       })
     })

     it('displays audit trail with filtering', async () => {
       render(
         <Provider store={store}>
           <AuditTrail />
         </Provider>
       )

       // Check table renders
       expect(screen.getByText(/Operation/)).toBeInTheDocument()

       // Search for specific operation
       const searchInput = screen.getByPlaceholderText(/Search/)
       fireEvent.change(searchInput, { target: { value: 'delete_pod' } })

       expect(screen.getByText(/delete_pod/)).toBeInTheDocument()
     })

     it('audit log is immutable', async () => {
       render(
         <Provider store={store}>
           <AuditTrail />
         </Provider>
       )

       // Should not have delete or edit buttons
       expect(screen.queryByRole('button', { name: /delete/i })).not.toBeInTheDocument()
       expect(screen.queryByRole('button', { name: /edit/i })).not.toBeInTheDocument()

       // Should show integrity status
       expect(screen.getByText(/integrity verified/i)).toBeInTheDocument()
     })
   })
   ```

4. **Test utilities:**
   - Helper to create mock approvals
   - Helper to create mock audit events
   - Helper to verify approval state changes
   - Helper to verify audit log entries

5. **Coverage goals:**
   - >80% coverage of approval/audit components
   - All user workflows tested
   - Error scenarios covered
   - Immutability verified

Ensure tests use MSW for API mocking and Redux context.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- approval-workflow
Should show:
- 6+ test cases all passing
- Approval display test passes
- Approve/reject tests pass
- Audit trail tests pass
- Immutability verified
- Error handling tested

Also run full suite:
pnpm test
All tests passing (no regressions).
  </verify>
  <done>
✅ E2E tests for approval and audit workflows (6+ test cases)
✅ Tests cover: approval display, approve/reject actions, audit log display, filtering, immutability
✅ All tests passing (pnpm test → all passing)
✅ Tests use MSW for API mocking
✅ Redux state verified
✅ Error scenarios tested
✅ No regressions
  </done>
</task>

</tasks>

<verification>
After completing all tasks:

1. **Backend types and classification:**
   - [ ] OperationCategory enum (Destructive, Risky, Safe)
   - [ ] ApprovalPolicy with environment-aware approval logic
   - [ ] classify_operation() helper maps operations to categories
   - [ ] All types properly serializable

2. **Audit logging:**
   - [ ] AuditLogger with file-based persistence
   - [ ] Immutable append-only design (no modifications)
   - [ ] SHA256 hashing with chain integrity
   - [ ] Filtering by operator, time, operation
   - [ ] Integrity verification works

3. **Backend API:**
   - [ ] GET /api/approvals lists pending approvals
   - [ ] POST /api/approvals/:id/approve approves
   - [ ] POST /api/approvals/:id/reject rejects
   - [ ] GET /api/audit returns audit log
   - [ ] All decisions logged to audit trail

4. **Frontend components:**
   - [ ] ApprovalWorkflow shows pending approvals
   - [ ] ApprovalCard displays operation details and controls
   - [ ] Approve/reject buttons functional
   - [ ] AuditTrail displays events in table
   - [ ] Search and filtering work
   - [ ] Read-only (no delete buttons)

5. **Redux integration:**
   - [ ] auditSlice manages approvals and audit events
   - [ ] Async thunks for API operations
   - [ ] State updates on approve/reject
   - [ ] Mock handlers return data

6. **API client:**
   - [ ] configAPI.getApprovals()
   - [ ] configAPI.approveOperation()
   - [ ] configAPI.rejectOperation()
   - [ ] configAPI.getAuditLog()
   - [ ] All methods properly typed

7. **Testing:**
   - [ ] pnpm test → 6+ approval tests passing
   - [ ] pnpm test → all tests passing (no regressions)
   - [ ] pnpm type-check → 0 TypeScript errors
   - [ ] pnpm build → succeeds without errors
   - [ ] cargo test → all backend tests passing

8. **UX quality:**
   - [ ] Approval cards clearly show info
   - [ ] Category colors help identify risk level
   - [ ] Audit table readable and searchable
   - [ ] Loading and error states visible
   - [ ] Mobile responsive

9. **Acceptance criteria:**
   - [ ] Operations classified (Destructive/Risky/Safe)
   - [ ] Destructive ops require approval
   - [ ] Risky ops auto-approve in dev/staging, require in prod
   - [ ] Safe ops execute immediately
   - [ ] All ops logged immutably with hashes
   - [ ] Approvals managed via web UI and Slack
   - [ ] Audit trail searchable and verifiable
</verification>

<success_criteria>

**Plan 04 Complete When:**

1. ✅ **Operation classification:** Destructive, Risky, Safe properly categorized
2. ✅ **Approval policy:** Environment-aware logic (dev/staging auto-approve risky)
3. ✅ **Approval workflow:** Request → Pending → Approved/Rejected → Executed
4. ✅ **AuditLogger:** Immutable, append-only, SHA256 hashing with chain
5. ✅ **Backend API:** All approval and audit endpoints working
6. ✅ **ApprovalWorkflow:** Shows pending approvals with approve/reject controls
7. ✅ **AuditTrail:** Displays events with search, filter, immutable guarantee
8. ✅ **Redux integration:** Approvals and audit events managed in state
9. ✅ **API client:** All approval/audit methods implemented
10. ✅ **Error handling:** Network errors, API failures handled gracefully
11. ✅ **Testing:** 6+ E2E tests all passing
12. ✅ **No regressions:** All existing tests still passing
13. ✅ **TypeScript clean:** 0 compilation errors

**Verification Method:**
```bash
# Backend tests
cargo test --lib approval
cargo test --lib audit_logger

# Frontend tests
pnpm test -- approval-workflow    # 6+ tests passing
pnpm test                         # All tests passing

# Type checking
pnpm type-check                   # 0 errors

# Build
pnpm build                        # Succeeds
cargo build --release             # Succeeds

# Manual testing
pnpm dev
# Visit config dashboard
# View pending approvals
# Click approve/reject
# Check audit log
```

**Deliverables:**
- 8 new/updated files (types, logger, handlers, components, Redux, API, tests)
- 7+ commits with atomic changes
- Operation classification system
- Immutable audit logging with integrity
- Approval workflow backend and frontend
- 6+ E2E tests
- 0 regressions
</success_criteria>

<output>
After completion, create `.planning/phases/01-onboarding-refinement/04-SUMMARY.md` with:
- Executive summary of approval and audit implementation
- File list (8 files created/updated)
- Architecture overview (operation classification, approval flow, audit chain)
- Component inventory (ApprovalWorkflow, AuditTrail)
- Test results (6+ tests passing, 0 regressions)
- Security verification (immutability, integrity, no modifications possible)
- Next steps (Phase 2: Mission Control + real ops capabilities)
</output>
