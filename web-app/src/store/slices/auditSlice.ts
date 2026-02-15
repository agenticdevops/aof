import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { configAPI } from '@/api/config'
import type { ApprovalRequest, AuditEvent } from '@/types/audit'

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
        decided_by: 'current_user',
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
