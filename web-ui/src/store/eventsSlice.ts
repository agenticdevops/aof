/**
 * Redux slice for coordination events.
 * Manages event stream from Phase 1 WebSocket connection.
 */

import { createSlice } from '@reduxjs/toolkit';
import type { PayloadAction } from '@reduxjs/toolkit';
import type { CoordinationEvent } from '../types/events';

/**
 * Events slice state.
 */
interface EventsState {
  /** Event array (limited to last 500) */
  events: CoordinationEvent[];

  /** Last received event ID */
  lastEventId: string;

  /** WebSocket connection status */
  connected: boolean;
}

/**
 * Initial state.
 */
const initialState: EventsState = {
  events: [],
  lastEventId: '',
  connected: false,
};

/**
 * Events slice with reducers.
 */
const eventsSlice = createSlice({
  name: 'events',
  initialState,
  reducers: {
    /**
     * Add event to array, keeping last 500.
     */
    addEvent: (state, action: PayloadAction<CoordinationEvent>) => {
      state.events.push(action.payload);
      state.lastEventId = action.payload.event_id;

      // Keep only last 500 events
      if (state.events.length > 500) {
        state.events = state.events.slice(-500);
      }
    },

    /**
     * Clear all events.
     */
    clearEvents: (state) => {
      state.events = [];
      state.lastEventId = '';
    },

    /**
     * Set connection status.
     */
    setConnected: (state, action: PayloadAction<boolean>) => {
      state.connected = action.payload;
    },
  },
});

export const { addEvent, clearEvents, setConnected } = eventsSlice.actions;

/**
 * Select all introduction events.
 */
export const selectIntroductionEvents = (state: { events: EventsState }) =>
  state.events.events.filter((e) => e.introduction !== undefined);

/**
 * Select introduction events for a specific agent.
 */
export const selectAgentIntroductionEvents = (
  state: { events: EventsState },
  agentId: string,
) =>
  state.events.events.filter(
    (e) => e.introduction !== undefined && e.introduction.agent_id === agentId,
  );

export default eventsSlice.reducer;
