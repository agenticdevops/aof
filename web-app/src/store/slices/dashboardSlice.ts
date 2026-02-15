import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { DashboardState, DashboardAgent } from '@/types/dashboard'
import type { RootState } from '../store'
import { useAppSelector } from '../hooks'

/**
 * Initial dashboard state
 */
const initialState: DashboardState = {
  agents: [],
  selectedAgent: null,
  isLoading: false,
  error: null,
}

/**
 * Dashboard slice - manages Mission Control agent grid state
 *
 * Note: WebSocket integration will be added in Plan 03.
 * This slice currently manages local state and mock data.
 */
export const dashboardSlice = createSlice({
  name: 'dashboard',
  initialState,
  reducers: {
    /**
     * Set the list of agents to display in the grid
     */
    setAgents: (state, action: PayloadAction<DashboardAgent[]>) => {
      state.agents = action.payload
      state.error = null
    },

    /**
     * Set the currently selected agent for detail modal
     */
    setSelectedAgent: (state, action: PayloadAction<DashboardAgent | null>) => {
      state.selectedAgent = action.payload
    },

    /**
     * Set loading state (for future async operations)
     */
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },

    /**
     * Set error state
     */
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
      state.isLoading = false
    },

    /**
     * Update a single agent's status and metrics
     */
    updateAgent: (state, action: PayloadAction<DashboardAgent>) => {
      const index = state.agents.findIndex(a => a.id === action.payload.id)
      if (index !== -1) {
        state.agents[index] = action.payload
      }

      // Update selected agent if it's the same one
      if (state.selectedAgent?.id === action.payload.id) {
        state.selectedAgent = action.payload
      }
    },

    /**
     * Clear all agents and reset state
     */
    clearDashboard: (state) => {
      state.agents = []
      state.selectedAgent = null
      state.error = null
      state.isLoading = false
    },
  },
})

// Export actions
export const {
  setAgents,
  setSelectedAgent,
  setLoading,
  setError,
  updateAgent,
  clearDashboard,
} = dashboardSlice.actions

// Export reducer
export default dashboardSlice.reducer

// Selectors
export const selectDashboardAgents = (state: RootState) => state.dashboard.agents
export const selectSelectedAgent = (state: RootState) => state.dashboard.selectedAgent
export const selectDashboardLoading = (state: RootState) => state.dashboard.isLoading
export const selectDashboardError = (state: RootState) => state.dashboard.error

/**
 * Custom hooks for dashboard state
 */

/**
 * Get all dashboard state
 */
export const useDashboard = () => {
  return useAppSelector((state) => state.dashboard)
}

/**
 * Get dashboard agents array
 */
export const useDashboardAgents = () => {
  return useAppSelector(selectDashboardAgents)
}

/**
 * Get selected agent for detail modal
 */
export const useSelectedAgent = () => {
  return useAppSelector(selectSelectedAgent)
}
