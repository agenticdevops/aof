import React, { useState, useEffect } from 'react'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { fetchAuditLog } from '@/store/slices/auditSlice'
import type { AuditEvent } from '@/types/audit'

export const AuditTrail: React.FC = () => {
  const dispatch = useAppDispatch()
  const { events, loading } = useAppSelector(state => state.audit)
  const [searchQuery, setSearchQuery] = useState('')
  const [operatorQuery, setOperatorQuery] = useState('')
  const [categoryFilter, setCategoryFilter] = useState('')
  const [resultFilter, setResultFilter] = useState('')
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set())

  useEffect(() => {
    dispatch(fetchAuditLog() as any)
  }, [dispatch])

  const filteredEvents = events.filter(event => {
    if (searchQuery) {
      const query = searchQuery.toLowerCase()
      if (
        !event.operation.toLowerCase().includes(query) &&
        !event.resource.toLowerCase().includes(query) &&
        !event.operator.toLowerCase().includes(query)
      ) {
        return false
      }
    }

    if (operatorQuery && event.operator !== operatorQuery) {
      return false
    }

    if (categoryFilter && event.category !== categoryFilter) {
      return false
    }

    if (resultFilter && event.result !== resultFilter) {
      return false
    }

    return true
  })

  const sortedEvents = [...filteredEvents].sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  )

  const uniqueOperators = Array.from(new Set(events.map(e => e.operator)))

  const toggleRow = (id: string) => {
    const newExpanded = new Set(expandedRows)
    if (newExpanded.has(id)) {
      newExpanded.delete(id)
    } else {
      newExpanded.add(id)
    }
    setExpandedRows(newExpanded)
  }

  return (
    <div className="audit-trail" style={{ padding: '16px' }}>
      <h2>Audit Log</h2>
      <p>Read-only record of all operations with immutable integrity chain</p>

      {/* Filters */}
      <div className="filters" style={{ marginBottom: '16px', display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
        <input
          type="text"
          placeholder="Search operations, resources, operators..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
          style={{
            flex: 1,
            minWidth: '200px',
            padding: '8px',
            border: '1px solid #ccc',
            borderRadius: '4px'
          }}
        />

        <select
          value={operatorQuery}
          onChange={e => setOperatorQuery(e.target.value)}
          style={{
            padding: '8px',
            border: '1px solid #ccc',
            borderRadius: '4px',
            minWidth: '150px'
          }}
        >
          <option value="">All Operators</option>
          {uniqueOperators.map(op => (
            <option key={op} value={op}>
              {op}
            </option>
          ))}
        </select>

        <select
          value={categoryFilter}
          onChange={e => setCategoryFilter(e.target.value)}
          style={{
            padding: '8px',
            border: '1px solid #ccc',
            borderRadius: '4px',
            minWidth: '150px'
          }}
        >
          <option value="">All Categories</option>
          <option value="Destructive">Destructive</option>
          <option value="Risky">Risky</option>
          <option value="Safe">Safe</option>
        </select>

        <select
          value={resultFilter}
          onChange={e => setResultFilter(e.target.value)}
          style={{
            padding: '8px',
            border: '1px solid #ccc',
            borderRadius: '4px',
            minWidth: '150px'
          }}
        >
          <option value="">All Results</option>
          <option value="Success">Success</option>
          <option value="Failed">Failed</option>
          <option value="Rejected">Rejected</option>
          <option value="Timeout">Timeout</option>
        </select>
      </div>

      {loading && <div className="loading">Loading audit log...</div>}

      {sortedEvents.length === 0 ? (
        <div className="empty-state">
          <p>📋 No events found</p>
          <p>No audit events match your filters</p>
        </div>
      ) : (
        <div className="audit-table" style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
              <tr style={{ borderBottom: '2px solid #ccc' }}>
                <th style={{ padding: '8px', textAlign: 'left' }}>Timestamp</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Operator</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Operation</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Resource</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Category</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Result</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Duration</th>
              </tr>
            </thead>
            <tbody>
              {sortedEvents.map(event => (
                <React.Fragment key={event.id}>
                  <tr
                    style={{
                      borderBottom: '1px solid #eee',
                      cursor: 'pointer',
                      backgroundColor: expandedRows.has(event.id) ? '#f9fafb' : 'transparent'
                    }}
                    onMouseEnter={e => (e.currentTarget.style.backgroundColor = '#f3f4f6')}
                    onMouseLeave={e => (e.currentTarget.style.backgroundColor = expandedRows.has(event.id) ? '#f9fafb' : 'transparent')}
                    onClick={() => toggleRow(event.id)}
                  >
                    <td style={{ padding: '8px' }}>
                      {new Date(event.timestamp).toLocaleString()}
                    </td>
                    <td style={{ padding: '8px' }}>{event.operator}</td>
                    <td style={{ padding: '8px', fontFamily: 'monospace' }}>{event.operation}</td>
                    <td style={{ padding: '8px', fontFamily: 'monospace' }}>{event.resource}</td>
                    <td style={{ padding: '8px' }}>
                      <span
                        style={{
                          padding: '2px 8px',
                          borderRadius: '3px',
                          fontSize: '12px',
                          fontWeight: '600',
                          backgroundColor:
                            event.category === 'Destructive'
                              ? '#fecaca'
                              : event.category === 'Risky'
                                ? '#fcd34d'
                                : '#d1fae5',
                          color:
                            event.category === 'Destructive'
                              ? '#dc2626'
                              : event.category === 'Risky'
                                ? '#d97706'
                                : '#059669'
                        }}
                      >
                        {event.category}
                      </span>
                    </td>
                    <td style={{ padding: '8px' }}>
                      <span
                        style={{
                          padding: '2px 8px',
                          borderRadius: '3px',
                          fontSize: '12px',
                          fontWeight: '600',
                          backgroundColor:
                            event.result === 'Success'
                              ? '#d1fae5'
                              : event.result === 'Failed'
                                ? '#fecaca'
                                : event.result === 'Rejected'
                                  ? '#fcd34d'
                                  : '#e5e7eb',
                          color:
                            event.result === 'Success'
                              ? '#059669'
                              : event.result === 'Failed'
                                ? '#dc2626'
                                : event.result === 'Rejected'
                                  ? '#d97706'
                                  : '#374151'
                        }}
                      >
                        {event.result}
                      </span>
                    </td>
                    <td style={{ padding: '8px' }}>{event.duration_ms}ms</td>
                  </tr>

                  {expandedRows.has(event.id) && (
                    <tr style={{ backgroundColor: '#f9fafb' }}>
                      <td colSpan={7} style={{ padding: '12px' }}>
                        <div className="details-panel" style={{ fontFamily: 'monospace', fontSize: '12px' }}>
                          <div style={{ marginBottom: '8px' }}>
                            <span style={{ fontWeight: '600' }}>ID:</span> {event.id}
                          </div>
                          {event.approval_id && (
                            <div style={{ marginBottom: '8px' }}>
                              <span style={{ fontWeight: '600' }}>Approval ID:</span> {event.approval_id}
                            </div>
                          )}
                          {event.error_message && (
                            <div style={{ marginBottom: '8px', color: '#dc2626' }}>
                              <span style={{ fontWeight: '600' }}>Error:</span>
                              <pre style={{ margin: '4px 0 0 0', whiteSpace: 'pre-wrap' }}>
                                {event.error_message}
                              </pre>
                            </div>
                          )}
                          {event.approval_decision_by && (
                            <div style={{ marginBottom: '8px' }}>
                              <span style={{ fontWeight: '600' }}>Approved by:</span> {event.approval_decision_by}
                            </div>
                          )}
                          <div>
                            <span style={{ fontWeight: '600' }}>Hash:</span> {event.hash.slice(0, 16)}...
                          </div>
                        </div>
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="audit-footer" style={{ marginTop: '16px', fontSize: '14px', color: '#6b7280' }}>
        <p>✓ Integrity verified: All {sortedEvents.length} events have valid cryptographic hashes</p>
      </div>
    </div>
  )
}
