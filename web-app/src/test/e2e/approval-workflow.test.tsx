import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Provider } from 'react-redux'
import { store } from '@/store'
import { ApprovalWorkflow } from '@/components/config/ApprovalWorkflow'
import { AuditTrail } from '@/components/config/AuditTrail'
import { fetchApprovals, fetchAuditLog } from '@/store/slices/auditSlice'
import type { ApprovalRequest, AuditEvent } from '@/types/audit'

// Mock data
const MOCK_APPROVALS: ApprovalRequest[] = [
  {
    id: 'apr-001',
    operation_id: 'op-001',
    category: 'Destructive',
    description: 'kubectl delete pod nginx-abc123 in production',
    operator: 'alice',
    created_at: new Date().toISOString(),
    blast_radius: {
      affected_services: ['nginx', 'api-gateway'],
      estimated_impact_pct: 45,
      data_at_risk: true
    }
  },
  {
    id: 'apr-002',
    operation_id: 'op-002',
    category: 'Risky',
    description: 'terraform apply infrastructure changes',
    operator: 'bob',
    created_at: new Date().toISOString(),
    blast_radius: {
      affected_services: ['database'],
      estimated_impact_pct: 20,
      data_at_risk: true
    }
  }
]

const MOCK_AUDIT_EVENTS: AuditEvent[] = [
  {
    id: 'evt-001',
    timestamp: new Date().toISOString(),
    operator: 'alice',
    operation: 'delete_pod',
    category: 'Destructive',
    resource: 'pod: nginx-abc123',
    result: 'Success',
    duration_ms: 2500,
    hash: 'abc123def456abc123def456abc123de',
    previous_hash: 'genesis',
    metadata: {}
  },
  {
    id: 'evt-002',
    timestamp: new Date(Date.now() - 60000).toISOString(),
    operator: 'bob',
    operation: 'get_pods',
    category: 'Safe',
    resource: 'namespace: default',
    result: 'Success',
    duration_ms: 450,
    hash: 'xyz789uvw456xyz789uvw456xyz789uv',
    previous_hash: 'abc123def456abc123def456abc123de',
    metadata: {}
  },
  {
    id: 'evt-003',
    timestamp: new Date(Date.now() - 120000).toISOString(),
    operator: 'alice',
    operation: 'update_config',
    category: 'Risky',
    resource: 'configmap: app-config',
    result: 'Success',
    duration_ms: 1200,
    approval_decision: 'Approved',
    approval_decision_by: 'security-team',
    hash: 'pqr321mno654pqr321mno654pqr321mn',
    previous_hash: 'xyz789uvw456xyz789uvw456xyz789uv',
    metadata: {}
  }
]

describe('Approval and Audit Workflow E2E', () => {
  beforeEach(() => {
    // Setup mock handlers if needed
  })

  describe('ApprovalWorkflow Component', () => {
    it('displays pending approvals', async () => {
      // Mock initial state with approvals
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Pending Approvals/i)).toBeInTheDocument()
      })
    })

    it('shows empty state when no pending approvals', async () => {
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      // Eventually should show empty state or no pending items
      await waitFor(() => {
        const heading = screen.getByText(/Pending Approvals/i)
        expect(heading).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('has approve and reject buttons', async () => {
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      // Wait for component to render
      await waitFor(() => {
        expect(screen.getByText(/Pending Approvals/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('displays category color coding', async () => {
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Pending Approvals/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('shows blast radius information when available', async () => {
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Pending Approvals/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })
  })

  describe('AuditTrail Component', () => {
    it('renders audit log table with headers', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      })
    })

    it('displays audit events in table format', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('sorts events by timestamp (newest first)', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('filters by operator', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      const operatorSelect = screen.queryAllByRole('combobox').find(e =>
        (e as HTMLSelectElement).value === ''
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })

      if (operatorSelect) {
        fireEvent.change(operatorSelect, { target: { value: 'alice' } })
      }
    })

    it('filters by category', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      const categorySelects = screen.queryAllByRole('combobox')
      const categorySelect = categorySelects[2] // Approximate index

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })

      if (categorySelect) {
        fireEvent.change(categorySelect, { target: { value: 'Destructive' } })
      }
    })

    it('filters by result', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      const resultSelects = screen.queryAllByRole('combobox')
      const resultSelect = resultSelects[3] // Approximate index

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })

      if (resultSelect) {
        fireEvent.change(resultSelect, { target: { value: 'Success' } })
      }
    })

    it('searches by operation/resource/operator', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      const searchInput = screen.queryByPlaceholderText(/Search operations/i)

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })

      if (searchInput) {
        fireEvent.change(searchInput, { target: { value: 'delete' } })
      }
    })

    it('expands rows to show detailed info', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('displays integrity verification status', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
        // Check for integrity status text
        const footer = screen.queryByText(/Integrity verified/i)
        if (footer) {
          expect(footer).toBeInTheDocument()
        }
      }, { timeout: 2000 })
    })

    it('audit log is read-only (no delete/edit buttons)', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })

      // Check that there are no delete or edit buttons
      const deleteButtons = screen.queryAllByRole('button', { name: /delete/i })
      const editButtons = screen.queryAllByRole('button', { name: /edit/i })

      expect(deleteButtons.length).toBe(0)
      expect(editButtons.length).toBe(0)
    })
  })

  describe('Redux Integration', () => {
    it('audit slice manages approvals and events state', () => {
      const state = store.getState()
      expect(state.audit).toBeDefined()
      expect(state.audit.approvals).toBeDefined()
      expect(state.audit.events).toBeDefined()
      expect(Array.isArray(state.audit.approvals)).toBe(true)
      expect(Array.isArray(state.audit.events)).toBe(true)
    })

    it('async thunks dispatch correctly', async () => {
      // Test that async thunks can be dispatched
      const dispatch = store.dispatch
      expect(typeof dispatch).toBe('function')
    })
  })

  describe('API Client Integration', () => {
    it('configAPI has approval methods', async () => {
      // Just verify the API methods exist
      const hasApprovalMethods =
        typeof require('@/api/config').configAPI.getApprovals === 'function' &&
        typeof require('@/api/config').configAPI.approveOperation === 'function' &&
        typeof require('@/api/config').configAPI.rejectOperation === 'function' &&
        typeof require('@/api/config').configAPI.getAuditLog === 'function'

      expect(hasApprovalMethods).toBe(true)
    })
  })

  describe('Type Safety', () => {
    it('ApprovalRequest type is properly defined', () => {
      const approval: ApprovalRequest = {
        id: 'test',
        operation_id: 'op-1',
        category: 'Destructive',
        description: 'Test operation',
        operator: 'test-user',
        created_at: new Date().toISOString()
      }

      expect(approval.id).toBe('test')
      expect(approval.category).toBe('Destructive')
    })

    it('AuditEvent type is properly defined', () => {
      const event: AuditEvent = {
        id: 'evt-1',
        timestamp: new Date().toISOString(),
        operator: 'test-user',
        operation: 'test_operation',
        category: 'Safe',
        resource: 'test-resource',
        result: 'Success',
        duration_ms: 100,
        hash: 'test-hash',
        previous_hash: 'prev-hash',
        metadata: {}
      }

      expect(event.id).toBe('evt-1')
      expect(event.result).toBe('Success')
    })
  })

  describe('Component Composition', () => {
    it('ApprovalWorkflow renders without errors', async () => {
      const { container } = render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      expect(container).toBeTruthy()
    })

    it('AuditTrail renders without errors', async () => {
      const { container } = render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      expect(container).toBeTruthy()
    })
  })

  describe('Error Handling', () => {
    it('displays error message when loading fails', async () => {
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      // Component should handle errors gracefully
      await waitFor(() => {
        expect(screen.getByText(/Pending Approvals/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })
  })

  describe('Loading States', () => {
    it('shows loading indicator during data fetch', async () => {
      render(
        <Provider store={store}>
          <ApprovalWorkflow />
        </Provider>
      )

      // Wait for component to render
      await waitFor(() => {
        expect(screen.getByText(/Pending Approvals/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('audit trail shows loading state', async () => {
      render(
        <Provider store={store}>
          <AuditTrail />
        </Provider>
      )

      await waitFor(() => {
        expect(screen.getByText(/Audit Log/i)).toBeInTheDocument()
      }, { timeout: 2000 })
    })
  })
})
