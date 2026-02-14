/**
 * Redux slice for coordination state management.
 * Manages agent health, standup results, and coordination metrics.
 */

import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import type {
  AgentHealthRecord,
  StandupResult,
  StandupResponseRecord,
  CoordinationMetrics,
  CoordinationState,
} from '../types/coordination';

/**
 * Initial coordination state.
 */
const initialState: CoordinationState = {
  health: [],
  latestStandup: null,
  metrics: null,
  isLoading: false,
  error: null,
  coordinationEnabled: false,
};

/**
 * Coordination slice with reducers for health, standup, and metrics.
 */
export const coordinationSlice = createSlice({
  name: 'coordination',
  initialState,
  reducers: {
    /**
     * Set complete agent health array (from API /api/coordination/health).
     */
    setHealth: (state, action: PayloadAction<AgentHealthRecord[]>) => {
      state.health = action.payload;
      state.coordinationEnabled = true;
      state.error = null;
    },

    /**
     * Update single agent health record (from WebSocket HeartbeatResponse).
     * Creates new record if agent not in state, otherwise updates existing.
     */
    updateAgentHealth: (state, action: PayloadAction<AgentHealthRecord>) => {
      const idx = state.health.findIndex(
        (a) => a.agent_id === action.payload.agent_id
      );

      if (idx >= 0) {
        state.health[idx] = action.payload;
      } else {
        state.health.push(action.payload);
      }
    },

    /**
     * Set latest standup result (from API /api/coordination/standup/latest).
     */
    setLatestStandup: (state, action: PayloadAction<StandupResult>) => {
      state.latestStandup = action.payload;
      state.error = null;
    },

    /**
     * Add standup response to latest standup (from WebSocket StandupResponse).
     * Only adds if latestStandup exists, otherwise no-op.
     */
    addStandupResponse: (state, action: PayloadAction<StandupResponseRecord>) => {
      if (state.latestStandup) {
        // Check if response already exists (avoid duplicates)
        const exists = state.latestStandup.responses.some(
          (r) => r.agent_id === action.payload.agent_id
        );

        if (!exists) {
          state.latestStandup.responses.push(action.payload);
        }
      }
    },

    /**
     * Update standup summary (from WebSocket StandupSummary).
     */
    updateStandupSummary: (
      state,
      action: PayloadAction<{ request_id: string; summary: string }>
    ) => {
      if (
        state.latestStandup &&
        state.latestStandup.request_id === action.payload.request_id
      ) {
        state.latestStandup.summary = action.payload.summary;
      }
    },

    /**
     * Set coordination metrics (from API /api/coordination/metrics).
     */
    setMetrics: (state, action: PayloadAction<CoordinationMetrics>) => {
      state.metrics = action.payload;
      state.error = null;
    },

    /**
     * Set loading state for async operations.
     */
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload;
    },

    /**
     * Set error message.
     */
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
      state.isLoading = false;
    },

    /**
     * Reset coordination state to initial values.
     */
    resetCoordination: (state) => {
      state.health = [];
      state.latestStandup = null;
      state.metrics = null;
      state.isLoading = false;
      state.error = null;
      state.coordinationEnabled = false;
    },
  },
});

/**
 * Export actions.
 */
export const {
  setHealth,
  updateAgentHealth,
  setLatestStandup,
  addStandupResponse,
  updateStandupSummary,
  setMetrics,
  setLoading,
  setError,
  resetCoordination,
} = coordinationSlice.actions;

/**
 * Export reducer.
 */
export default coordinationSlice.reducer;

/**
 * Selectors for accessing coordination state.
 */
export const selectCoordinationHealth = (state: { coordination: CoordinationState }) =>
  state.coordination.health;

export const selectLatestStandup = (state: { coordination: CoordinationState }) =>
  state.coordination.latestStandup;

export const selectCoordinationMetrics = (state: { coordination: CoordinationState }) =>
  state.coordination.metrics;

export const selectCoordinationLoading = (state: { coordination: CoordinationState }) =>
  state.coordination.isLoading;

export const selectCoordinationError = (state: { coordination: CoordinationState }) =>
  state.coordination.error;

export const selectCoordinationEnabled = (state: { coordination: CoordinationState }) =>
  state.coordination.coordinationEnabled;
