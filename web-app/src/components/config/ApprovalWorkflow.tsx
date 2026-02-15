import React, { useState, useEffect } from 'react'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { fetchApprovals, approveOperation, rejectOperation } from '@/store/slices/auditSlice'
import type { ApprovalRequest } from '@/types/audit'

export const ApprovalWorkflow: React.FC = () => {
  const dispatch = useAppDispatch()
  const { approvals, loading, error } = useAppSelector(state => state.audit)
  const [actionLoading, setActionLoading] = useState<{ [key: string]: 'approve' | 'reject' | null }>({})

  useEffect(() => {
    dispatch(fetchApprovals() as any)
    const interval = setInterval(() => {
      dispatch(fetchApprovals() as any)
    }, 10000)
    return () => clearInterval(interval)
  }, [dispatch])

  const pendingApprovals = approvals.filter(a => !a.decision)

  const handleApprove = async (id: string) => {
    setActionLoading(prev => ({ ...prev, [id]: 'approve' }))
    try {
      await dispatch(approveOperation({ id, reason: 'Approved via web UI' }) as any)
    } finally {
      setActionLoading(prev => ({ ...prev, [id]: null }))
    }
  }

  const handleReject = async (id: string) => {
    setActionLoading(prev => ({ ...prev, [id]: 'reject' }))
    try {
      await dispatch(rejectOperation({ id, reason: 'Rejected via web UI' }) as any)
    } finally {
      setActionLoading(prev => ({ ...prev, [id]: null }))
    }
  }

  return (
    <div className="approval-workflow">
      <h2>Pending Approvals</h2>

      {loading && <div className="loading">Loading approvals...</div>}
      {error && <div className="error">{error}</div>}

      {pendingApprovals.length === 0 ? (
        <div className="empty-state">
          <p>✅ No pending approvals</p>
          <p>All operations have been approved or are executing</p>
        </div>
      ) : (
        <div className="approvals-list">
          {pendingApprovals.map(approval => (
            <ApprovalCard
              key={approval.id}
              approval={approval}
              onApprove={() => handleApprove(approval.id)}
              onReject={() => handleReject(approval.id)}
              isLoading={!!actionLoading[approval.id]}
              loadingAction={actionLoading[approval.id]}
            />
          ))}
        </div>
      )}
    </div>
  )
}

interface ApprovalCardProps {
  approval: ApprovalRequest
  onApprove: () => Promise<void>
  onReject: () => Promise<void>
  isLoading: boolean
  loadingAction: 'approve' | 'reject' | null
}

const ApprovalCard: React.FC<ApprovalCardProps> = ({
  approval,
  onApprove,
  onReject,
  isLoading,
  loadingAction
}) => {
  const categoryColorMap: Record<string, string> = {
    Destructive: '#ef4444',
    Risky: '#f59e0b',
    Safe: '#10b981'
  }
  const categoryColor = categoryColorMap[approval.category] || '#6b7280'

  const categoryBgMap: Record<string, string> = {
    Destructive: '#fecaca',
    Risky: '#fcd34d',
    Safe: '#d1fae5'
  }
  const categoryBg = categoryBgMap[approval.category] || '#e5e7eb'

  const categoryTextMap: Record<string, string> = {
    Destructive: '#dc2626',
    Risky: '#d97706',
    Safe: '#059669'
  }
  const categoryText = categoryTextMap[approval.category] || '#374151'

  return (
    <div className="approval-card" style={{ borderLeftColor: categoryColor }}>
      <div className="card-header">
        <h3>Operation Approval Required</h3>
        <span
          className="category-badge"
          style={{
            backgroundColor: categoryBg,
            color: categoryText,
            padding: '4px 12px',
            borderRadius: '4px',
            fontSize: '12px',
            fontWeight: '600'
          }}
        >
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
              <p className="warning" style={{ color: '#ef4444' }}>
                ⚠️ Data at risk
              </p>
            )}
          </div>
        )}
      </div>

      <div className="card-actions" style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
        <button
          onClick={onApprove}
          disabled={isLoading}
          className="btn-approve"
          style={{
            flex: 1,
            padding: '8px 16px',
            backgroundColor: '#10b981',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            opacity: isLoading ? 0.6 : 1
          }}
        >
          {loadingAction === 'approve' ? 'Approving...' : '✓ Approve'}
        </button>
        <button
          onClick={onReject}
          disabled={isLoading}
          className="btn-reject"
          style={{
            flex: 1,
            padding: '8px 16px',
            backgroundColor: '#ef4444',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            opacity: isLoading ? 0.6 : 1
          }}
        >
          {loadingAction === 'reject' ? 'Rejecting...' : '✗ Reject'}
        </button>
      </div>
    </div>
  )
}
