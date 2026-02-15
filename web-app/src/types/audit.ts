export type OperationCategory = 'Destructive' | 'Risky' | 'Safe'
export type ApprovalStatus = 'Pending' | 'Approved' | 'Rejected'
export type OperationResult = 'Success' | 'Failed' | 'Rejected' | 'Timeout'

export interface BlastRadius {
  affected_services: string[]
  estimated_impact_pct: number
  data_at_risk: boolean
}

export interface ApprovalDecision {
  status: ApprovalStatus
  timestamp: string
  decided_by: string
  reason?: string
}

export interface ApprovalRequest {
  id: string
  operation_id: string
  category: OperationCategory
  description: string
  operator: string
  blast_radius?: BlastRadius
  created_at: string
  decision?: ApprovalDecision
  decided_at?: string
  decided_by?: string
  reason?: string
}

export interface AuditEvent {
  id: string
  timestamp: string
  operator: string
  operation: string
  category: OperationCategory
  resource: string
  approval_id?: string
  approval_decision?: ApprovalStatus
  result: OperationResult
  error_message?: string
  duration_ms: number
  approval_decision_by?: string
  approval_decision_at?: string
  metadata: Record<string, unknown>
  hash: string
  previous_hash: string
}
